//! Build script for `fbink-sys`.
//!
//! Responsibilities:
//!   1. Fetch a pinned FBInk source snapshot, plus the few submodules that are
//!      actually needed to build it (`font8x8`, `stb` and `libunibreak`), and
//!      `i2c-tools` for Kobo targets. Everything is cached under the target
//!      directory.
//!   2. Compile FBInk into a static library with `cc`, honoring the crate's
//!      feature flags (mapped to FBInk's `FBINK_WITH_*` defines) and the
//!      requested target platform.
//!   3. Generate Rust bindings from FBInk's public `fbink.h` via `bindgen`.
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
