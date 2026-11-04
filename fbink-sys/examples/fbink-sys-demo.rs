//! Demo of the raw `fbink-sys` FFI bindings driving a real eInk framebuffer.
//!
//! Everything here is plain `unsafe` FFI against FBInk's C API (see `fbink.h`),
//! 1:1 with what the generated bindings expose: `FBInkConfig` is a plain struct
//! you zero and fill in, return codes are checked by hand.
//!
//! Run it on a device (or an emulated one) with an eInk framebuffer. Static
//! musl cross-build with zig as the C toolchain (see `zig-cc-arm` below):
//!
//! ```sh
//! env CC_armv7-unknown-linux-musleabihf=zig-cc-arm \
//!     CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=zig-cc-arm \
//!     RUSTFLAGS="-C link-self-contained=no" FBINK_TARGET=kindle \
//!     cargo build -p fbink-sys --example fbink-sys-demo --target armv7-unknown-linux-musleabihf
//! scp target/armv7-unknown-linux-musleabihf/debug/examples/fbink-sys-demo root@device:/mnt/us/
//! ```
//!
//! (`zig-cc-arm` = `zig cc -target arm-linux-musleabihf`; bindgen additionally
//! needs `BINDGEN_EXTRA_CLANG_ARGS_armv7-unknown-linux-musleabihf` pointing at
//! zig's musl include dirs so it doesn't trip over the host's glibc headers.)
//!
//! It prints FBInk build info to stdout, then renders a small showcase:
//! device info, a few fonts, a grayscale ramp, an inverted accent box, a
//! progress bar, and a final flashing refresh. On exit it restores the screen
//! snapshot taken at startup (`fbink_dump`/`fbink_restore`), so the device's
//! own UI is left exactly as it was — no manual redraw needed.

use fbink_sys::*;
use std::ffi::{CStr, CString};
use std::io::IsTerminal;
use std::mem::zeroed;
use std::process::exit;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// How long the demo stays on screen before it exits on its own (seconds).
/// Overridable: `fbink-sys-demo [SECONDS]`.
const DEFAULT_EXIT_AFTER: u64 = 30;

/// Exit mechanism: wait for Enter on a terminal, or until the timeout expires
/// when stdin is not a terminal (e.g. plain `ssh host cmd`). Either way this
/// returns and the caller can clean up and exit.
fn wait_for_exit(seconds: u64) {
    if std::io::stdin().is_terminal() {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            let _ = tx.send(());
        });
        println!("Press Enter to exit (auto-exit in {seconds}s)...");
        let _ = rx.recv_timeout(Duration::from_secs(seconds));
    } else {
        println!("stdin is not a terminal; auto-exit in {seconds}s...");
        thread::sleep(Duration::from_secs(seconds));
    }
}

/// Check an FBInk return code: negative values are `-errno`-style failures.
fn ck(rv: i32, what: &str) -> i32 {
    if rv < 0 {
        eprintln!("error: {what} failed with rv={rv}");
        exit(1);
    }
    rv
}

/// A zeroed `FBInkConfig` is FBInk's "sane defaults" configuration.
fn default_cfg() -> FBInkConfig {
    // SAFETY: all-zero is a valid FBInkConfig (all `bool` fields are `false`).
    unsafe { zeroed::<FBInkConfig>() }
}

fn print(fbfd: i32, text: &str, cfg: &FBInkConfig) {
    let c = CString::new(text).expect("text contains a NUL byte");
    // SAFETY: `c` outlives the call; `cfg` is a valid FBInkConfig.
    ck(unsafe { fbink_print(fbfd, c.as_ptr(), cfg) }, "fbink_print");
}

fn fill_rect(fbfd: i32, cfg: &FBInkConfig, rect: &FBInkRect, gray: u8) {
    // SAFETY: pointers are valid for the duration of the call.
    ck(
        unsafe { fbink_fill_rect_gray(fbfd, cfg, rect, false, gray) },
        "fbink_fill_rect_gray",
    );
}

fn main() {
    println!("=== fbink-sys raw FFI demo ===");

    // Build-time info of the compiled FBInk.
    // SAFETY: fbink_version() returns a static NUL-terminated string.
    let version = unsafe { CStr::from_ptr(fbink_version()) };
    println!("FBInk version : {}", version.to_string_lossy());
    // SAFETY: plain scalar getters.
    println!("FBInk target  : {}", unsafe { fbink_target() });
    println!("FBInk features: {:#010x}", unsafe { fbink_features() });

    // Open the framebuffer: FBFD_AUTO lets FBInk pick the right device.
    // SAFETY: no pointers involved.
    let fbfd = ck(unsafe { fbink_open() }, "fbink_open");
    println!("framebuffer fd: {fbfd}");

    let cfg = default_cfg();
    // SAFETY: `cfg` is a valid FBInkConfig.
    ck(unsafe { fbink_init(fbfd, &cfg) }, "fbink_init");

    // Query the device/screen state.
    // SAFETY: `state` is written to by FBInk; zeroed is fine to start from.
    let mut state: FBInkState = unsafe { zeroed() };
    // SAFETY: both pointers are valid.
    unsafe { fbink_get_state(&cfg, &mut state) };

    let cstr = |p: *const ::std::os::raw::c_char| -> String {
        if p.is_null() {
            String::from("<none>")
        } else {
            // SAFETY: FBInk hands us NUL-terminated strings.
            unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    };
    let arr = |a: &[::std::os::raw::c_char]| -> String {
        let p = a.as_ptr();
        if p.is_null() {
            String::from("<none>")
        } else {
            // SAFETY: the array is NUL-terminated by FBInk.
            unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
        }
    };

    println!(
        "device        : {} ({})",
        arr(&state.device_name),
        arr(&state.device_codename)
    );
    println!("platform      : {}", arr(&state.device_platform));
    println!(
        "view          : {}x{} @ {}bpp, dpi {}",
        state.view_width, state.view_height, state.bpp, state.screen_dpi
    );
    println!(
        "grid          : {}x{} cells, font {}x{} (x{})",
        state.max_cols, state.max_rows, state.font_w, state.font_h, state.fontsize_mult
    );
    println!("font name     : {}", cstr(state.font_name));

    // Snapshot the screen as we found it, so we can put it back on exit.
    // (Clearing to white would leave the Kindle framework's UI invisible until
    // the next input event makes it repaint.)
    let mut dump: FBInkDump = unsafe { zeroed() };
    // SAFETY: `dump` is zeroed; fbink_dump allocates the pixel buffer itself.
    let have_dump = unsafe { fbink_dump(fbfd, &mut dump) } >= 0;
    if !have_dump {
        eprintln!("warning: fbink_dump failed; the screen will be cleared on exit");
    }

    // ---------------------------------------------------------------- drawing
    let w = state.view_width as i32;
    let h = state.view_height as i32;

    // Start from a blank screen (NULL rect + no_rota=false = full screen).
    // SAFETY: NULL rect is explicitly allowed here.
    ck(
        unsafe { fbink_cls(fbfd, &cfg, std::ptr::null(), false) },
        "fbink_cls",
    );

    // Big centered title (fontmult 2, the BLOCK font).
    let mut title = default_cfg();
    title.row = 1;
    title.is_centered = true;
    title.fontmult = 2;
    title.fontname = FONT_INDEX_E::BLOCK;
    print(fbfd, "fbink-sys", &title);

    let mut sub = default_cfg();
    sub.row = 4;
    sub.is_centered = true;
    print(fbfd, "raw FFI demo: Rust -> C FBInk", &sub);

    // Left-aligned device info, one line per font flavor.
    let mut info = default_cfg();
    info.row = 7;
    info.col = 1;
    print(
        fbfd,
        &format!(
            "device: {} / {} on {}",
            arr(&state.device_name),
            arr(&state.device_codename),
            arr(&state.device_platform)
        ),
        &info,
    );
    info.row += 1;
    print(
        fbfd,
        &format!(
            "screen: {}x{} 8bpp {}dpi, {}x{} cells",
            state.view_width, state.view_height, state.screen_dpi, state.max_cols, state.max_rows
        ),
        &info,
    );

    let mut sci = default_cfg();
    sci.row = 10;
    sci.col = 1;
    sci.fontname = FONT_INDEX_E::SCIENTIFICA;
    print(fbfd, "font: scientifica 0123456789", &sci);

    let mut ter = default_cfg();
    ter.row = 12;
    ter.col = 1;
    ter.fontname = FONT_INDEX_E::TERMINUS;
    print(fbfd, "font: terminus", &ter);

    // Grayscale ramp: 16 swatches, pure `fbink_fill_rect_gray` pixel work.
    let margin = 40i32;
    let ramp_h = h / 6;
    let ramp_top = h / 2 - ramp_h / 2;
    let swatch_w = (w - 2 * margin) / 16;
    for i in 0..16i32 {
        let rect = FBInkRect {
            left: (margin + i * swatch_w) as u16,
            top: ramp_top as u16,
            width: (swatch_w - 4) as u16,
            height: ramp_h as u16,
        };
        fill_rect(fbfd, &cfg, &rect, (i * 17) as u8);
    }
    // Frame the ramp in black.
    let frame = FBInkRect {
        left: (margin - 8) as u16,
        top: (ramp_top - 8) as u16,
        width: (w - 2 * margin + 16) as u16,
        height: (ramp_h + 16) as u16,
    };
    for edge in [
        FBInkRect {
            width: frame.width,
            height: 4,
            ..frame
        },
        FBInkRect {
            top: (frame.top as i32 + ramp_h + 12) as u16,
            width: frame.width,
            height: 4,
            ..frame
        },
        FBInkRect {
            width: 4,
            height: frame.height,
            ..frame
        },
        FBInkRect {
            left: (frame.left as i32 + frame.width as i32 - 4) as u16,
            width: 4,
            height: frame.height,
            ..frame
        },
    ] {
        fill_rect(fbfd, &cfg, &edge, 0);
    }

    let mut ramp_label = default_cfg();
    ramp_label.row = ((ramp_top + ramp_h + 24) as usize / state.font_h as usize) as i16;
    ramp_label.col = 1;
    print(fbfd, "fbink_fill_rect_gray(): 16-step ramp", &ramp_label);

    // Inverted accent box (`fbink_invert_rect`).
    let accent = FBInkRect {
        left: margin as u16,
        top: (ramp_top + ramp_h + 24 + state.font_h as i32 * 2) as u16,
        width: (w - 2 * margin) as u16,
        height: state.font_h as u16 * 2,
    };
    // SAFETY: valid rect, no_rota=false.
    ck(
        unsafe { fbink_invert_rect(fbfd, &accent, false) },
        "fbink_invert_rect",
    );
    let mut accent_label = default_cfg();
    accent_label.row =
        ((accent.top as i32 + accent.height as i32 + 8) as usize / state.font_h as usize) as i16;
    accent_label.col = 1;
    print(fbfd, "fbink_invert_rect()", &accent_label);

    // Progress bar in the lower third.
    let mut bar = default_cfg();
    bar.row = ((h * 3 / 4) as usize / state.font_h as usize) as i16;
    bar.col = 2;
    bar.is_centered = true;
    // SAFETY: scalar percentage + config.
    ck(
        unsafe { fbink_print_progress_bar(fbfd, 75, &bar) },
        "fbink_print_progress_bar",
    );

    // Footer.
    let mut foot = default_cfg();
    foot.row = (state.max_rows as i16) - 3;
    foot.is_centered = true;
    print(fbfd, "fbink-sys: GPL-3.0+ raw bindings", &foot);

    // ------------------------------------------------------------- the refresh
    // eInk only shows what you explicitly refresh: do a final flashing GC16.
    let mut refresh = default_cfg();
    refresh.wfm_mode = WFM_MODE_INDEX_E::WFM_GC16;
    refresh.is_flashing = true;
    // SAFETY: full-screen region (all zeros) + config.
    ck(
        unsafe { fbink_refresh(fbfd, 0, 0, 0, 0, &refresh) },
        "fbink_refresh",
    );
    // SAFETY: plain scalar marker.
    let marker = unsafe { fbink_get_last_marker() };
    println!("refresh marker: {marker}");
    ck(
        unsafe { fbink_wait_for_complete(fbfd, marker) },
        "fbink_wait_for_complete",
    );

    // What did we just draw? (device coords)
    // SAFETY: plain getter returning a struct.
    let last = unsafe { fbink_get_last_rect(false) };
    println!("last rect     : {:?}", last);

    // ------------------------------------------------------------- exit path
    // Exit mechanism: Enter to quit, or auto-exit after the timeout.
    let exit_after = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_EXIT_AFTER);
    wait_for_exit(exit_after);

    // Leave the screen exactly as we found it: restore the snapshot taken
    // before we drew anything. fbink_restore honors the config's refresh flags
    // (is_flashing / no_refresh) just like fbink_refresh.
    let mut bye = default_cfg();
    bye.wfm_mode = WFM_MODE_INDEX_E::WFM_GC16;
    if have_dump {
        // SAFETY: `dump` was filled by fbink_dump and is still owned by us.
        ck(unsafe { fbink_restore(fbfd, &bye, &dump) }, "fbink_restore");
        // SAFETY: frees the buffer allocated by fbink_dump; never call twice.
        ck(
            unsafe { fbink_free_dump_data(&mut dump) },
            "fbink_free_dump_data",
        );
    } else {
        // Fallback: blank screen + refresh (the UI then waits for an input
        // event before repainting itself).
        // SAFETY: NULL rect = full screen.
        ck(
            unsafe { fbink_cls(fbfd, &cfg, std::ptr::null(), false) },
            "fbink_cls (cleanup)",
        );
        // SAFETY: full-screen region + config.
        ck(
            unsafe { fbink_refresh(fbfd, 0, 0, 0, 0, &bye) },
            "fbink_refresh (cleanup)",
        );
    }
    // SAFETY: plain scalar marker.
    let marker = unsafe { fbink_get_last_marker() };
    ck(
        unsafe { fbink_wait_for_complete(fbfd, marker) },
        "fbink_wait_for_complete (cleanup)",
    );

    // SAFETY: fd was returned by fbink_open().
    ck(unsafe { fbink_close(fbfd) }, "fbink_close");
    println!("=== demo done ===");
    exit(0);
}
