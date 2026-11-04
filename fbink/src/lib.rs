//! Safe, idiomatic Rust bindings to [FBInk](https://github.com/NiLuJe/FBInk)
//! ("FrameBuffer eInker"), a library to print text & images to eInk Linux
//! framebuffers (Kobo, Kindle, Cervantes, reMarkable, PocketBook, generic
//! Linux fb).
//!
//! This is the safe wrapper; the raw 1:1 FFI is re-exported as [`sys`] for the
//! low-level escape hatches ([`Fbink::fb_pointer`], [`Fbink::get_fb_info`],
//! ...).
//!
//! # Quickstart
//!
//! ```no_run
//! use fbink::{Config, Fbink, Font, LAST_MARKER};
//!
//! # fn main() -> fbink::Result<()> {
//! let fb = Fbink::open()?;                              // or Fbink::auto()
//! fb.init(&Config::new().quiet(true))?;                 // once, in the main thread
//! fb.print("Hello, world!", &Config::new().row(2).font(Font::Unscii))?;
//! fb.refresh(0, 0, 0, 0, &Config::new())?;              // full-screen refresh
//! fb.wait_for_complete(LAST_MARKER)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! Cargo features mirror FBInk's own `MINIMAL=1 DRAW=1 BITMAP=1 ...` build
//! flags (with FBInk's own implications), and are forwarded to `fbink-sys`:
//!
//! | Feature | FBInk define | What it enables |
//! |---|---|---|
//! | `draw` | `FBINK_WITH_DRAW` | drawing primitives |
//! | `bitmap` | `FBINK_WITH_BITMAP` | fixed-cell text + base font (implies `draw`) |
//! | `fonts` | `FBINK_WITH_FONTS` | all bundled fonts (implies `bitmap`) |
//! | `unifont` | `FBINK_WITH_UNIFONT` | GNU Unifont (implies `fonts`, +2MB!) |
//! | `image` | `FBINK_WITH_IMAGE` | images & dumps (implies `draw`) |
//! | `opentype` | `FBINK_WITH_OPENTYPE` | TrueType/OpenType text (implies `draw`) |
//! | `input` | `FBINK_WITH_INPUT` | input device scanning |
//! | `button-scan` | `FBINK_WITH_BUTTON_SCAN` | deprecated Kobo USBMS scanner (implies `draw`) |
//! | `serde` | — | `Serialize`/`Deserialize` on the public types |
//!
//! Everything is compiled regardless (the default features reproduce a
//! standard full-featured FBInk build): calling into a disabled feature simply
//! fails at runtime with [`Error::NotSupported`]. The exceptions are the
//! [`ot`] & [`input`] modules, which are only compiled with their feature.
//!
//! Beyond the FBInk feature map, the fieldless enums implement `Display` &
//! `FromStr` round-tripping their variant names (`"Gc16"`, `"UnsciiAlt"`, ...)
//! as well as `EnumIter`/`EnumCount`, courtesy of `strum`.
//!
//! # Platform
//!
//! The *device* flavor is selected at build time via the `FBINK_TARGET`
//! environment variable (`linux` = default, `kobo`, `kindle`, `kindle-legacy`,
//! `cervantes`, `remarkable`, `pocketbook`); the resulting code only ever runs
//! on Linux. Calls that need eInk hardware support fail with
//! [`Error::NotSupported`] elsewhere (e.g. [`Fbink::refresh`] on a plain
//! Linux build).
//!
//! # Threading
//!
//! [`Fbink::init`] is *not* thread-safe (it sets global state): call it from
//! your main thread before spawning threads. Everything else is thread-safe.
//!
//! # License
//!
//! FBInk is GPL-3.0-or-later and statically linked: the resulting binary is
//! subject to the same terms.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod config;
mod dump;
mod enums;
mod error;
mod fb;
#[cfg(feature = "input")]
#[cfg_attr(docsrs, doc(cfg(feature = "input")))]
pub mod input;
#[cfg(feature = "opentype")]
#[cfg_attr(docsrs, doc(cfg(feature = "opentype")))]
pub mod ot;
mod rect;
mod state;

pub use config::Config;
pub use dump::Dump;
pub use enums::{
    Align, BgColor, CfaMode, FgColor, Font, FontStyle, HwDither, MtkHalftone, MtkSwipeDirection,
    NtxRotaQuirk, Padding, PixelFormat, SunxiForceRota, Target, WaveformMode,
};
pub use error::{Error, Result};
pub use fb::{
    Fbink, Features, KEEP_CURRENT_BITDEPTH, KEEP_CURRENT_GRAYSCALE, KEEP_CURRENT_ROTATE,
    LAST_MARKER, PackedPixel, ReinitChanges, TOGGLE_GRAYSCALE,
};
pub use rect::Rect;
pub use state::State;

// The raw FFI, re-exported for the low-level escape hatches (e.g. the
// `fb_var_screeninfo` returned by `Fbink::get_fb_info`).
pub use fbink_sys as sys;

use error::check;

/// The version of the loaded FBInk library.
pub fn version() -> &'static str {
    // Safety: fbink_version() returns a pointer to a static C string.
    unsafe { std::ffi::CStr::from_ptr(sys::fbink_version()) }
        .to_str()
        .unwrap_or("")
}

/// The platform this FBInk library was built for.
pub fn target() -> Target {
    // Safety: plain call.
    let raw = unsafe { sys::fbink_target() };
    Target::from_raw(raw).unwrap_or(Target::Linux)
}

/// The feature set compiled into this FBInk library.
pub fn features() -> Features {
    // Safety: plain call.
    Features::from_raw(unsafe { sys::fbink_features() })
}

/// The update marker of the last refresh sent by this FBInk session
/// ([`LAST_MARKER`] when there was none).
pub fn get_last_marker() -> u32 {
    // Safety: plain call.
    unsafe { sys::fbink_get_last_marker() }
}

/// The coordinates & dimensions of the last thing that was *drawn*
/// (empty when nothing was).
///
/// With `rotated`, returns rotated coordinates on quirky (Kobo @ 16bpp)
/// framebuffer states; the default unrotated ones match the rest of the API.
pub fn get_last_rect(rotated: bool) -> Rect {
    // Safety: plain call.
    Rect::from_raw(unsafe { sys::fbink_get_last_rect(rotated) })
}

/// Update FBInk's internal verbosity flags (`is_verbose`, `is_quiet` &
/// `to_syslog` from `cfg`) without a full re-init.
pub fn update_verbosity(cfg: &Config) {
    // Safety: cfg is a valid reference.
    unsafe { sys::fbink_update_verbosity(cfg.raw()) };
}

/// Update FBInk's internal pen colors (`fg_color` & `bg_color` from `cfg`)
/// without a full re-init.
pub fn update_pen_colors(cfg: &Config) -> Result<()> {
    // Safety: cfg is a valid reference.
    check(unsafe { sys::fbink_update_pen_colors(cfg.raw()) })
}

/// Set the foreground pen to an 8-bit luminance value.
///
/// If `update` is set and that's already the current color, aborts early and
/// returns `true` ("already the same"). With `quantize`, rounds to the nearest
/// eInk palette color.
pub fn set_fg_pen_gray(y: u8, quantize: bool, update: bool) -> Result<bool> {
    // Safety: plain call.
    pen_result(unsafe { sys::fbink_set_fg_pen_gray(y, quantize, update) })
}

/// Set the background pen to an 8-bit luminance value.
/// See [`set_fg_pen_gray`].
pub fn set_bg_pen_gray(y: u8, quantize: bool, update: bool) -> Result<bool> {
    // Safety: plain call.
    pen_result(unsafe { sys::fbink_set_bg_pen_gray(y, quantize, update) })
}

/// Set the foreground pen to an RGBA color (`a` opaque is `0xFF`).
///
/// NOTE: subsequent (re)inits *will* reset the pen colors to a grayscale
/// representation of these values. See [`set_fg_pen_gray`] for the flags.
pub fn set_fg_pen_rgba(r: u8, g: u8, b: u8, a: u8, quantize: bool, update: bool) -> Result<bool> {
    // Safety: plain call.
    pen_result(unsafe { sys::fbink_set_fg_pen_rgba(r, g, b, a, quantize, update) })
}

/// Set the background pen to an RGBA color (`a` opaque is `0xFF`).
/// See [`set_fg_pen_rgba`].
pub fn set_bg_pen_rgba(r: u8, g: u8, b: u8, a: u8, quantize: bool, update: bool) -> Result<bool> {
    // Safety: plain call.
    pen_result(unsafe { sys::fbink_set_bg_pen_rgba(r, g, b, a, quantize, update) })
}

/// Pack an 8-bit luminance value into a pixel for the *current* framebuffer
/// pixel format (for use with [`Fbink::put_pixel`] & [`Fbink::fill_rect`]).
///
/// Re-pack after a bitdepth change (e.g. on a non-empty [`Fbink::reinit`]).
pub fn pack_pixel_gray(y: u8) -> Result<PackedPixel> {
    let mut px: u32 = 0;
    // Safety: px is a valid out-pointer.
    check(unsafe { sys::fbink_pack_pixel_gray(y, &mut px) })?;
    Ok(PackedPixel(px))
}

/// Pack an RGBA color into a pixel for the *current* framebuffer pixel format.
/// See [`pack_pixel_gray`].
pub fn pack_pixel_rgba(r: u8, g: u8, b: u8, a: u8) -> Result<PackedPixel> {
    let mut px: u32 = 0;
    // Safety: px is a valid out-pointer.
    check(unsafe { sys::fbink_pack_pixel_rgba(r, g, b, a, &mut px) })?;
    Ok(PackedPixel(px))
}

/// Forcefully wake up the EPDC (Kobo Mk. 8+ only; [`Error::NotSupported`]
/// elsewhere).
pub fn wakeup_epdc() -> Result<()> {
    // Safety: plain call.
    check(unsafe { sys::fbink_wakeup_epdc() })
}

/// Whether the device appears to be in a quirky framebuffer state that may
/// require a re-init to produce sane results.
///
/// Deprecated by FBInk in favor of [`Fbink::reinit`], which is smarter.
pub fn is_fb_quirky() -> bool {
    // Safety: plain call.
    unsafe { sys::fbink_is_fb_quirky() }
}

/// Set up the swipe animation direction & duration used by every refresh with
/// `animated` set in the config (Kindle MTK; `steps` max is 60 on Bellatrix).
pub fn mtk_set_swipe_data(direction: MtkSwipeDirection, steps: u8) -> Result<()> {
    // Safety: plain call.
    check(unsafe { sys::fbink_mtk_set_swipe_data(direction.to_raw(), steps) })
}

/// Map the `set_*_pen_*` return convention: `OK_ALREADY_SAME` on early abort.
fn pen_result(ret: i32) -> Result<bool> {
    check(ret)?;
    Ok(ret as u32 == sys::OK_ALREADY_SAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Link & sanity smoke test (no framebuffer needed).
    #[test]
    fn global_entry_points_work() {
        assert!(!version().is_empty());
        assert_eq!(
            target(),
            Target::from_raw(unsafe { sys::fbink_target() }).unwrap()
        );
        // A MINIMAL build has zero features; a default build has plenty.
        let feats = features();
        assert!(feats.contains(Features::MINIMAL));
        // No refresh was ever sent from this session.
        assert_eq!(get_last_marker(), LAST_MARKER);
    }

    #[test]
    fn pen_flags_roundtrip() {
        // Not a real assertion about behavior (no fb here): just make sure the
        // mapping stays consistent with OK_ALREADY_SAME's value.
        assert_eq!(pen_result(sys::OK_ALREADY_SAME as i32), Ok(true));
        assert_eq!(pen_result(0), Ok(false));
    }
}
