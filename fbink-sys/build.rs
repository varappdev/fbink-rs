//! Build script for `fbink-sys`.
//!
//! Responsibilities:
//!   1. Fetch a pinned FBInk source snapshot, plus the few submodules that are
//!      actually needed to build it (`font8x8`, `stb` and `libunibreak`), and
//!      `i2c-tools` for Kobo targets. Everything is cached under the target
//!      directory.
//!   2. Apply the local patches in `patches/` to the FBInk source (also when
//!      built from `FBINK_SOURCE_DIR`, which is modified in place) as strict
//!      exact-text splices of their unified-diff hunks. Patches that no longer
//!      apply fail the build loudly; see `patches/README.md`.
//!   3. Compile FBInk into a static library with `cc`, honoring the crate's
//!      feature flags (mapped to FBInk's `FBINK_WITH_*` defines) and the
//!      requested target platform.
//!   4. Generate Rust bindings from FBInk's public `fbink.h` via `bindgen`.
//!
//! Environment variables:
//!   * `FBINK_TARGET`: target platform, one of `linux` (default), `kobo`,
//!     `kindle`, `kindle-legacy`, `cervantes`, `remarkable`, `pocketbook`.
//!     NOTE: this selects FBInk's *device* target, it has nothing to do with
//!     the Rust target triple: you can only ever build this for Linux targets.
//!   * `FBINK_SOURCE_DIR`: path to an existing FBInk checkout (e.g. cloned with
//!     `--recursive`) to build from, instead of downloading the pinned snapshot.
//!   * `FBINK_FETCH`: command used to download sources (`curl` or `wget`);
//!     auto-detected when unset.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

// The exact FBInk revision these bindings are tethered to (master).
// NOTE: FBInk's API is not frozen on master: when bumping this, diff `fbink.h`
//       and adjust the safe wrapper accordingly.
const FBINK_COMMIT: &str = "886f25f13368859ad8a899b88d04c26e19cda32e";
// Local patches applied to the FBInk source before compiling (plain unified
// diffs in `patches/`, see `patches/README.md`). Fix known upstream bugs;
// when bumping the pin, each patch must either keep applying or be rebased /
// dropped: anything else fails the build loudly.
const PATCHES: &[&str] = &["fix-fbink_state_dump-varargs-non-kobo.patch"];
// The handful of submodules actually required to build the library.
const FONT8X8_COMMIT: &str = "fa74831b9076805271cdedc0ee12137aa548e524";
const STB_COMMIT: &str = "dfa32facac4a16184dd66f8e482bf15a678f2bf3";
const UNIBREAK_COMMIT: &str = "3ce4bfa3129ff3738046a44a6db533d2ce25af2b";
const I2C_TOOLS_COMMIT: &str = "ec1267371e378fc6aaf1cd7686b5b9b31293255a";

const FBINK_URL: &str = "https://codeload.github.com/NiLuJe/FBInk/tar.gz";
const FONT8X8_URL: &str = "https://codeload.github.com/NiLuJe/font8x8/tar.gz";
const STB_URL: &str = "https://codeload.github.com/NiLuJe/stb/tar.gz";
const UNIBREAK_URL: &str = "https://codeload.github.com/adah1972/libunibreak/tar.gz";
// i2c-tools lives on git.kernel.org, but is mirrored on GitHub, which keeps
// everything on plain codeload tarballs (cgit's snapshot endpoint blocks
// non-browser user agents).
const I2C_TOOLS_URL: &str =
    "https://codeload.github.com/kernelorg-mirror/utils_i2c-tools_i2c-tools/tar.gz";

fn main() {
    println!("cargo:rerun-if-env-changed=FBINK_TARGET");
    println!("cargo:rerun-if-env-changed=FBINK_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=FBINK_FETCH");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=patches");

    // Feature flags (as enabled on this crate), resolved with FBInk's own
    // feature implications (c.f., FBInk's Makefile).
    let unifont = feature("unifont");
    let fonts = feature("fonts") || unifont;
    let bitmap = feature("bitmap") || fonts;
    let image = feature("image");
    let opentype = feature("opentype");
    let input = feature("input");
    let button_scan = feature("button-scan");
    let draw = feature("draw") || bitmap || image || opentype || button_scan;

    // Target platform (FBInk flavor, *not* the Rust target triple).
    let (target, target_is_kobo) = match fbink_target().as_str() {
        "linux" => ("FBINK_FOR_LINUX", false),
        "kobo" => ("FBINK_FOR_KOBO", true),
        "kindle" => ("FBINK_FOR_KINDLE", false),
        "kindle-legacy" | "legacy" => ("FBINK_FOR_KINDLE", false),
        "cervantes" => ("FBINK_FOR_CERVANTES", false),
        "remarkable" => ("FBINK_FOR_REMARKABLE", false),
        "pocketbook" => ("FBINK_FOR_POCKETBOOK", false),
        other => panic!(
            "fbink-sys: unknown FBINK_TARGET `{other}`, \
             expected one of: linux, kobo, kindle, kindle-legacy, cervantes, remarkable, pocketbook"
        ),
    };

    let cache = cache_dir();
    let (fbink_src, vendor_parent) = match env::var_os("FBINK_SOURCE_DIR") {
        Some(dir) => local_source(PathBuf::from(dir), &cache),
        None => fetched_sources(&cache),
    };
    apply_patches(&fbink_src);

    //
    // Compile FBInk.
    //
    let version_define = format!("\"{FBINK_COMMIT}\"");
    let mut build = cc::Build::new();
    build
        .include(&fbink_src)
        .include(&vendor_parent)
        .define("_GNU_SOURCE", None)
        .define("_REENTRANT", "1")
        .define("NDEBUG", None)
        .define("FBINK_MINIMAL", None)
        .define("FBINK_VERSION", Some(version_define.as_str()))
        .define(target, None)
        .warnings(false)
        .file(fbink_src.join("fbink.c"))
        .file(fbink_src.join("cutef8/utf8.c"))
        .file(fbink_src.join("cutef8/dfa.c"));

    if draw {
        build.define("FBINK_WITH_DRAW", None);
    }
    if bitmap {
        build.define("FBINK_WITH_BITMAP", None);
    }
    if fonts {
        build.define("FBINK_WITH_FONTS", None);
    }
    if unifont {
        build.define("FBINK_WITH_UNIFONT", None);
    }
    if image {
        build.define("FBINK_WITH_IMAGE", None);
        build.file(fbink_src.join("qimagescale/qimagescale.c"));
    }
    if opentype {
        build.define("FBINK_WITH_OPENTYPE", None);
        // Line-breaking support for the OpenType code path.
        let ub_src = pick_dir(&fbink_src, &vendor_parent, "libunibreak/src");
        for f in [
            "linebreak.c",
            "linebreakdata.c",
            "unibreakdef.c",
            "linebreakdef.c",
            "eastasianwidthdef.c",
        ] {
            build.file(ub_src.join(f));
        }
    }
    if input {
        build.define("FBINK_WITH_INPUT", None);
        // NOTE: fbink_input_scan.c is #included by fbink.c, no need to build it
        //       separately (that's only for FBInk's standalone libfbink_input).
    }
    if button_scan && target_is_kobo {
        build.define("FBINK_WITH_BUTTON_SCAN", None);
    }

    if target_is_kobo {
        // The Kobo rotation quirks code talks to the accelerometer over i2c.
        let i2c_src = pick_dir(&fbink_src, &vendor_parent, "i2c-tools");
        build.include(i2c_src.join("include"));
        build.file(i2c_src.join("lib/smbus.c"));
    }

    build.compile("fbink");

    // Extra system libraries.
    println!("cargo:rustc-link-lib=m");
    if fbink_target() == "pocketbook" {
        // PocketBook's InkView is dlopen'ed for device identification.
        println!("cargo:rustc-link-lib=dl");
    }

    //
    // Generate bindings.
    //
    let bindings = bindgen::Builder::default()
        .header(fbink_src.join("fbink.h").to_string_lossy())
        .clang_arg(format!("-I{}", fbink_src.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .generate_comments(true)
        .layout_tests(false)
        .allowlist_function("fbink_.*")
        .allowlist_type(
            "FBInk.*|.*_INDEX_E|.*_INDEX_T|FONT_STYLE_E|FONT_STYLE_T|DEVICE_ID_T|FBINK_TARGET_E|\
             INPUT_DEVICE_TYPE_E|INPUT_DEVICE_TYPE_T|INPUT_SETTINGS_TYPE_E|INPUT_SETTINGS_TYPE_T|\
             .*_DEVICE_ID_E|fb_var_screeninfo|fb_fix_screeninfo",
        )
        .allowlist_var(
            "FBINK_.*|FBFD_AUTO|LAST_MARKER|DEVICE_UNKNOWN|OK_.*|KEEP_CURRENT_.*|TOGGLE_GRAYSCALE|\
             SCAN_ONLY|OPEN_BLOCKING|MATCH_ALL|EXCLUDE_ALL|NO_RECAP",
        )
        .generate()
        .expect("fbink-sys: unable to generate bindings from fbink.h");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("fbink-sys: couldn't write bindings.rs");
}

/// Whether a cargo feature is enabled on this crate.
fn feature(name: &str) -> bool {
    env::var_os(format!(
        "CARGO_FEATURE_{}",
        name.to_uppercase().replace('-', "_")
    ))
    .is_some()
}

/// The FBInk platform to build for (not the Rust target triple!).
fn fbink_target() -> String {
    match env::var("FBINK_TARGET") {
        Ok(t) => t.to_lowercase(),
        Err(_) => {
            println!(
                "cargo:warning=fbink-sys: FBINK_TARGET is not set, defaulting to `linux` \
                 (no eInk refresh support). Set FBINK_TARGET=kobo|kindle|kindle-legacy|cervantes|\
                 remarkable|pocketbook for your device."
            );
            "linux".to_owned()
        }
    }
}

/// Apply the local patches in `patches/` to the FBInk source tree.
///
/// Patches are plain unified diffs for a single file. Each hunk's exact
/// preimage must occur exactly once in the target file and is replaced by its
/// postimage; a hunk whose postimage is already in place is skipped (the
/// source cache outlives rebuilds). Ambiguous matches, missing context or
/// unparseable diffs abort the build loudly. See `patches/README.md`.
fn apply_patches(fbink_src: &Path) {
    let patches_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set")).join("patches");
    for name in PATCHES {
        let patch_path = patches_dir.join(name);
        let patch = fs::read_to_string(&patch_path)
            .unwrap_or_else(|e| panic!("fbink-sys: cannot read local patch {}: {e}", patch_path.display()));
        let (rel_path, hunks) = parse_patch(&patch)
            .unwrap_or_else(|e| panic!("fbink-sys: cannot parse local patch {name}: {e}"));
        let target = fbink_src.join(&rel_path);
        let mut source = fs::read_to_string(&target)
            .unwrap_or_else(|e| panic!("fbink-sys: cannot read {}: {e}", target.display()));
        let mut changed = false;
        for (preimage, postimage) in &hunks {
            match source.matches(preimage.as_str()).count() {
                1 => {
                    source = source.replace(preimage.as_str(), postimage.as_str());
                    changed = true;
                }
                0 if source.matches(postimage.as_str()).count() == 1 => {
                    // Already applied: nothing to do.
                }
                0 => panic!(
                    "fbink-sys: local patch {name} no longer applies to {rel_path}.\n\
                     The FBInk pin was likely bumped (or the source was modified).\n\
                     Rebase the patch in fbink-sys/patches/ if the bug it fixes is still\n\
                     present upstream, or drop it if not (see fbink-sys/patches/README.md).\n\
                     When iterating on the patch itself, wipe the source cache first:\n\
                     `cargo clean` or `rm -rf target/*/build/fbink-sys-src`."
                ),
                n => panic!(
                    "fbink-sys: local patch {name} matches {n} places in {rel_path}; \
                     add context lines to its hunk"
                ),
            }
        }
        if changed {
            fs::write(&target, source)
                .unwrap_or_else(|e| panic!("fbink-sys: cannot write {}: {e}", target.display()));
            println!("cargo:warning=fbink-sys: applied local patch {name} to the FBInk source");
        }
    }
}

/// Parse the single file section of a unified diff into its target path and
/// its hunks as exact `(preimage, postimage)` line blocks.
fn parse_patch(patch: &str) -> Result<(String, Vec<(String, String)>), String> {
    let lines: Vec<&str> = patch.lines().collect();

    // File header: "--- a/<path>" followed by "+++ b/<path>".
    let mut i = 0;
    while i + 1 < lines.len() && !(lines[i].starts_with("--- ") && lines[i + 1].starts_with("+++ ")) {
        i += 1;
    }
    if i + 1 >= lines.len() {
        return Err("no `--- `/`+++ ` file header found".to_owned());
    }
    let name = lines[i + 1]
        .split_whitespace()
        .nth(1)
        .ok_or("empty file name in `+++ ` line")?;
    let rel_path = name.strip_prefix("b/").unwrap_or(name).to_owned();
    if rel_path.is_empty() || rel_path.starts_with('/') || rel_path.contains("..") {
        return Err(format!("refusing to patch `{rel_path}`"));
    }
    i += 2;

    // Hunks, consumed strictly by their declared line counts.
    let mut hunks = Vec::new();
    while i < lines.len() {
        let header = lines[i];
        if header.starts_with("--- ") {
            return Err("multi-file patches are not supported".to_owned());
        }
        if !header.starts_with("@@ ") {
            if hunks.is_empty() {
                return Err(format!("unexpected line before first hunk: {header:?}"));
            }
            i += 1; // Tolerate trailing noise (e.g. a git signature).
            continue;
        }
        let (old_count, new_count) = hunk_counts(header)?;
        i += 1;

        let (mut old, mut new) = (String::new(), String::new());
        let (mut oc, mut nc) = (0usize, 0usize);
        while (oc < old_count || nc < new_count) && i < lines.len() {
            let line = lines[i];
            i += 1;
            if line.starts_with('\\') {
                continue; // "\ No newline at end of file"
            }
            let (marker, text) = match line.chars().next() {
                Some(m @ (' ' | '-' | '+')) => (m, &line[1..]),
                None => (' ', ""), // Some tools emit context empty lines as "".
                _ => return Err(format!("unexpected line in hunk: {line:?}")),
            };
            match marker {
                ' ' if oc < old_count && nc < new_count => {
                    old.push_str(text);
                    old.push('\n');
                    new.push_str(text);
                    new.push('\n');
                    oc += 1;
                    nc += 1;
                }
                '-' if oc < old_count => {
                    old.push_str(text);
                    old.push('\n');
                    oc += 1;
                }
                '+' if nc < new_count => {
                    new.push_str(text);
                    new.push('\n');
                    nc += 1;
                }
                _ => return Err(format!("hunk line count mismatch: {line:?}")),
            }
        }
        if oc != old_count || nc != new_count {
            return Err("truncated hunk".to_owned());
        }
        hunks.push((old, new));
    }
    if hunks.is_empty() {
        return Err("no hunks found".to_owned());
    }
    Ok((rel_path, hunks))
}

/// The `(old, new)` line counts of a `@@ -a[,b] +c[,d] @@` hunk header.
fn hunk_counts(header: &str) -> Result<(usize, usize), String> {
    let mut ranges = header[3..].split_whitespace();
    let count = |range: Option<&str>| -> Result<usize, String> {
        let range = range.ok_or("missing range in hunk header")?;
        let rest = range
            .strip_prefix(['-', '+'])
            .ok_or(format!("bad hunk range `{range}`"))?;
        match rest.split_once(',') {
            Some((_, n)) => n.parse().map_err(|_| format!("bad hunk count in `{range}`")),
            None => Ok(1),
        }
    };
    Ok((count(ranges.next())?, count(ranges.next())?))
}

/// Cache directory for downloaded sources: `<target>/<profile>/build/fbink-sys-src`.
/// Shared between feature-set builds of this crate, but wiped by `cargo clean`.
fn cache_dir() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    // OUT_DIR is `<...>/build/<pkg-hash>/out`
    let cache = out_dir
        .parent()
        .and_then(Path::parent)
        .expect("unexpected OUT_DIR layout")
        .join("fbink-sys-src");
    fs::create_dir_all(&cache).expect("fbink-sys: couldn't create the source cache directory");
    cache
}

/// Use a user-provided FBInk checkout: it may already contain the needed
/// submodules (when cloned with `--recursive`); fetch whatever is missing.
fn local_source(dir: PathBuf, cache: &Path) -> (PathBuf, PathBuf) {
    let dir = dir.canonicalize().unwrap_or_else(|e| {
        panic!(
            "fbink-sys: cannot access FBINK_SOURCE_DIR {}: {e}",
            dir.display()
        )
    });
    if !dir.join("fbink.h").is_file() {
        panic!(
            "fbink-sys: FBINK_SOURCE_DIR ({}) does not look like an FBInk checkout",
            dir.display()
        );
    }
    let vendor = cache.join("src");
    fetch_submodules(&dir, &vendor);
    (dir, vendor)
}

/// Fetch the pinned FBInk snapshot + submodules; returns `(fbink_src, vendor_parent)`
/// where `vendor_parent` holds the submodules (`stb/`, `libunibreak/`, `i2c-tools/`).
fn fetched_sources(cache: &Path) -> (PathBuf, PathBuf) {
    let src = cache.join("src");
    let fbink = fetch_extract(
        "fbink",
        FBINK_COMMIT,
        &format!("{FBINK_URL}/{FBINK_COMMIT}"),
        &src,
    );
    fetch_submodules(&fbink, &src);
    (fbink, src)
}

/// Ensure `stb`, `libunibreak` & `i2c-tools` are available, either inside the
/// FBInk tree (recursive checkout) or extracted next to it.
fn fetch_submodules(fbink_dir: &Path, vendor: &Path) {
    for (name, url, commit) in [
        ("font8x8", FONT8X8_URL, FONT8X8_COMMIT),
        ("stb", STB_URL, STB_COMMIT),
        ("libunibreak", UNIBREAK_URL, UNIBREAK_COMMIT),
        ("i2c-tools", I2C_TOOLS_URL, I2C_TOOLS_COMMIT),
    ] {
        if !fbink_dir.join(name).join(".git").exists() && !has_content(&fbink_dir.join(name)) {
            fetch_extract(name, commit, &format!("{url}/{commit}"), vendor);
        }
    }
}

/// Does `dir` exist and contain at least one entry?
fn has_content(dir: &Path) -> bool {
    fs::read_dir(dir)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false)
}

/// Download & extract a codeload tarball into `parent/<name>`.
fn fetch_extract(name: &str, commit: &str, url: &str, parent: &Path) -> PathBuf {
    let dest = parent.join(name);
    let marker = dest.join(format!(".fbink-sys-fetched-{commit}"));
    if marker.is_file() {
        return dest;
    }

    fs::create_dir_all(parent).expect("fbink-sys: couldn't create the source directory");
    let dl_dir = parent.join(".downloads");
    fs::create_dir_all(&dl_dir).expect("fbink-sys: couldn't create the download cache");
    let tarball = dl_dir.join(format!("{name}-{commit}.tar.gz"));
    if !tarball.is_file() {
        download(url, &tarball);
    }

    // Extract to a scratch dir, then move the single top-level dir in place.
    let scratch = parent.join(format!(".extract-{name}"));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).expect("fbink-sys: couldn't create the extraction directory");
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(&scratch)
        .status()
        .expect("fbink-sys: `tar` is required to extract FBInk sources");
    if !status.success() {
        panic!("fbink-sys: failed to extract {}", tarball.display());
    }
    let extracted = fs::read_dir(&scratch)
        .expect("fbink-sys: unreadable extraction directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .expect("fbink-sys: unexpected tarball layout");

    let _ = fs::remove_dir_all(&dest);
    fs::rename(&extracted, &dest).unwrap_or_else(|e| {
        panic!(
            "fbink-sys: couldn't move {} into place: {e}",
            extracted.display()
        )
    });
    let _ = fs::remove_dir_all(&scratch);
    fs::write(&marker, b"").expect("fbink-sys: couldn't write the cache marker");
    dest
}

/// Download `url` to `dest` using curl or wget (or `$FBINK_FETCH`).
fn download(url: &str, dest: &Path) {
    let tmp = dest.with_extension("part");
    let err = match env::var_os("FBINK_FETCH") {
        Some(cmd) => {
            let s = Command::new(cmd).arg(url).arg(&tmp).status();
            Some(status_to_err(s))
        }
        None => {
            let curl = Command::new("curl")
                .args(["-fsSL", "--retry", "3", "-o"])
                .arg(&tmp)
                .arg(url)
                .status();
            match curl {
                Ok(s) if s.success() => None,
                _ => {
                    let wget = Command::new("wget")
                        .args(["-q", "-O"])
                        .arg(&tmp)
                        .arg(url)
                        .status();
                    Some(status_to_err(wget))
                }
            }
        }
    };
    if let Some(e) = err {
        panic!(
            "fbink-sys: could not download {url}: {e}\n\
             Install `curl` or `wget`, point FBINK_FETCH at a downloader \
             (invoked as `<cmd> <url> <output>`), or set FBINK_SOURCE_DIR to a \
             local FBInk checkout (e.g. `git clone --recursive https://github.com/NiLuJe/FBInk`)."
        );
    }
    fs::rename(&tmp, dest)
        .unwrap_or_else(|e| panic!("fbink-sys: couldn't move {} into place: {e}", tmp.display()));
}

fn status_to_err(status: io::Result<std::process::ExitStatus>) -> String {
    match status {
        Ok(s) if s.success() => String::new(),
        Ok(s) => format!("download failed with {s}"),
        Err(e) => format!("no downloader available ({e})"),
    }
}

/// Pick `fbink_src/<rel>` when it exists (recursive checkout), `<vendor>/<rel>` otherwise.
fn pick_dir(fbink_src: &Path, vendor: &Path, rel: &str) -> PathBuf {
    let in_tree = fbink_src.join(rel);
    if has_content(&in_tree) {
        in_tree
    } else {
        vendor.join(rel)
    }
}
