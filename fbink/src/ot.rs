//! TrueType/OpenType text rendering (`opentype` feature).
//!
//! [`OtConfig`] wraps FBInk's `FBInkOTConfig` together with its **v2** font
//! API (`fbink_add_ot_font_v2`/`fbink_free_ot_fonts_v2`): the set of loaded
//! fonts is tied to the config instance and released on [`Drop`]. The struct's
//! `font` field is FBInk-private: it stays `NULL` and is only ever managed by
//! FBInk itself.

use crate::config::Config;
use crate::enums::{FontStyle, Padding};
use crate::error::{check, cstring, Error, Result};
use crate::fb::Fbink;
use fbink_sys as sys;

/// A TrueType/OpenType font configuration (margins, style, size, ...).
///
/// At least one font must be loaded with [`OtConfig::add_font`] before
/// printing (FBInk answers [`Error::NoData`] otherwise). Fonts must be valid
/// `*.otf`/`*.ttf` files: FBInk will *segfault* on non-font or obfuscated
/// (e.g. stock Kobo) files.
#[derive(Debug)]
pub struct OtConfig {
    raw: sys::FBInkOTConfig,
}

impl OtConfig {
    /// A zeroed configuration: FBInk's documented sane defaults (regular
    /// style, 12pt, no margins).
    pub fn new() -> Self {
        // FBInkOTConfig MUST be zero-initialized (in particular the private
        // `font` pointer).
        Self {
            raw: unsafe { std::mem::zeroed() },
        }
    }

    /// Set the four margins (in pixels, relative to the top-left edge of the
    /// bounding box). Negative values count backwards from the opposite edge.
    pub fn margins(mut self, top: i16, bottom: i16, left: i16, right: i16) -> Self {
        self.raw.margins.top = top;
        self.raw.margins.bottom = bottom;
        self.raw.margins.left = left;
        self.raw.margins.right = right;
        self
    }

    /// Top margin in pixels (negative counts backwards from the bottom edge).
    pub fn margin_top(mut self, px: i16) -> Self {
        self.raw.margins.top = px;
        self
    }

    /// Bottom margin in pixels (supports negative values too).
    pub fn margin_bottom(mut self, px: i16) -> Self {
        self.raw.margins.bottom = px;
        self
    }

    /// Left margin in pixels (negative counts backwards from the right edge).
    pub fn margin_left(mut self, px: i16) -> Self {
        self.raw.margins.left = px;
        self
    }

    /// Right margin in pixels (supports negative values too).
    pub fn margin_right(mut self, px: i16) -> Self {
        self.raw.margins.right = px;
        self
    }

    /// Default font style used when `formatted` is off (defaults to Regular).
    pub fn style(mut self, style: FontStyle) -> Self {
        self.raw.style = style.to_raw();
        self
    }

    /// Size of the text in points (defaults to 12pt when unset).
    /// Superseded by [`OtConfig::size_px`].
    pub fn size_pt(mut self, pt: f32) -> Self {
        self.raw.size_pt = pt;
        self
    }

    /// Size of the text in pixels (takes precedence over
    /// [`OtConfig::size_pt`]).
    pub fn size_px(mut self, px: u16) -> Self {
        self.raw.size_px = px;
        self
    }

    /// Horizontal centering (relative to the printable area set by margins).
    pub fn centered(mut self, yes: bool) -> Self {
        self.raw.is_centered = yes;
        self
    }

    /// Pad the drawing area (paint it in the background color).
    ///
    /// Unlike the fixed-cell codepath, this always applies to *both* sides of
    /// the chosen axis (e.g. [`Padding::Horizontal`] prevents overlaps between
    /// consecutive strings on the same line).
    pub fn padding(mut self, padding: Padding) -> Self {
        self.raw.padding = padding.to_raw();
        self
    }

    /// Treat the string as "formatted" (markdown-like **bold**/*italic*).
    pub fn formatted(mut self, yes: bool) -> Self {
        self.raw.is_formatted = yes;
        self
    }

    /// Abort after the line-break computation pass (no rendering).
    ///
    /// NOTE: early enough that not *every* truncation case can be predicted.
    pub fn compute_only(mut self, yes: bool) -> Self {
        self.raw.compute_only = yes;
        self
    }

    /// Abort (as early as possible) instead of truncating when the string
    /// doesn't fit at the current font size ([`Error::NoSpace`]).
    ///
    /// Cannot prevent drawing truncated content in *every* case (broken font
    /// metrics): combine with `no_refresh` and a [`OtConfig::compute_only`]
    /// pass when computing a best-fit font size.
    pub fn no_truncation(mut self, yes: bool) -> Self {
        self.raw.no_truncation = yes;
        self
    }

    /// Load an OpenType font (`*.otf`/`*.ttf`) for use with
    /// [`Fbink::print_ot`], with the given style.
    ///
    /// Fonts may be replaced without freeing first; all of them are released
    /// when this [`OtConfig`] is dropped.
    pub fn add_font(&mut self, path: &str, style: FontStyle) -> Result<()> {
        let path = cstring(path)?;
        // Safety: path is a valid C string; raw is a valid, zero-initialized
        // (or FBInk-managed) FBInkOTConfig.
        check(unsafe { sys::fbink_add_ot_font_v2(path.as_ptr(), style.to_raw(), &mut self.raw) })
    }

    /// The raw `fbink-sys` configuration.
    pub fn raw(&self) -> &sys::FBInkOTConfig {
        &self.raw
    }
}

impl Default for OtConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OtConfig {
    fn drop(&mut self) {
        // Safety: raw is a valid FBInkOTConfig managed by the v2 API.
        // Safe even when no font was ever loaded (it just returns -EINVAL).
        unsafe { sys::fbink_free_ot_fonts_v2(&mut self.raw) };
    }
}

/// Line-breaking & fitting information for a [`Fbink::print_ot_fit`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OtFit {
    /// Expected amount of lines needed, according to font metrics.
    pub computed_lines: u16,
    /// Actually rendered amount of lines (stays 0 on early aborts).
    pub rendered_lines: u16,
    /// Bounding box of the string (at computation time, padding excluded).
    pub bbox: OtBBox,
    /// True if the string was truncated (at computation or rendering time).
    pub truncated: bool,
}

/// The width/height of an [`OtFit`]'s bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OtBBox {
    pub width: u16,
    pub height: u16,
}

impl OtFit {
    fn from_raw(raw: &sys::FBInkOTFit) -> Self {
        Self {
            computed_lines: raw.computed_lines,
            rendered_lines: raw.rendered_lines,
            bbox: OtBBox {
                width: raw.bbox.width,
                height: raw.bbox.height,
            },
            truncated: raw.truncated,
        }
    }
}

impl Fbink {
    /// Print a string using OpenType fonts.
    ///
    /// Positioning uses margins (in pixels) instead of rows/columns; on
    /// success, returns the new top margin for use in subsequent calls (`0`
    /// meaning no room is left at the current margins/font size).
    ///
    /// `cfg` is optional (honors `is_inverted`, `is_flashing`, `is_cleared`,
    /// `is_centered`, `is_halfway`, `is_overlay`, `is_fgless`, `is_bgless`,
    /// `fg_color`, `bg_color`, `valign`, `halign`, `wfm_mode`,
    /// `dithering_mode`, `is_nightmode` & `no_refresh`).
    ///
    /// Fails with [`Error::NoData`] when no font has been loaded,
    /// [`Error::OutOfRange`] on out-of-range margins, [`Error::NoSpace`] when
    /// `no_truncation` is set and the string doesn't fit.
    pub fn print_ot(&self, text: &str, ot: &OtConfig, cfg: Option<&Config>) -> Result<i32> {
        self.print_ot_impl(text, ot, cfg, None)
    }

    /// Like [`Fbink::print_ot`], but also returns the [`OtFit`] line-breaking
    /// information (e.g. to compute a best-fit font size for N lines).
    pub fn print_ot_fit(
        &self,
        text: &str,
        ot: &OtConfig,
        cfg: Option<&Config>,
    ) -> Result<(i32, OtFit)> {
        let mut fit: sys::FBInkOTFit = unsafe { std::mem::zeroed() };
        let margin = self.print_ot_impl(text, ot, cfg, Some(&mut fit))?;
        Ok((margin, OtFit::from_raw(&fit)))
    }

    fn print_ot_impl(
        &self,
        text: &str,
        ot: &OtConfig,
        cfg: Option<&Config>,
        fit: Option<&mut sys::FBInkOTFit>,
    ) -> Result<i32> {
        let text = cstring(text)?;
        let cfg_ptr = cfg.map_or(std::ptr::null(), |c| c.raw() as *const sys::FBInkConfig);
        let fit_ptr = fit.map_or(std::ptr::null_mut(), |f| f as *mut sys::FBInkOTFit);
        // Safety: text is a valid C string; ot is a valid reference; cfg_ptr
        // and fit_ptr are null or valid.
        let ret =
            unsafe { sys::fbink_print_ot(self.fd(), text.as_ptr(), ot.raw(), cfg_ptr, fit_ptr) };
        if ret < 0 {
            Err(Error::from_ret(ret))
        } else {
            // Positive (or zero) = the new top margin.
            Ok(ret)
        }
    }
}

/// Load an OpenType font into FBInk's *global* font set (the legacy v1 API).
///
/// Prefer [`OtConfig::add_font`], which ties the fonts to one config instance;
/// if you use this, you **must** call [`free_ot_fonts`] when done.
pub fn add_ot_font(path: &str, style: FontStyle) -> Result<()> {
    let path = cstring(path)?;
    // Safety: path is a valid C string.
    check(unsafe { sys::fbink_add_ot_font(path.as_ptr(), style.to_raw()) })
}

/// Free all fonts loaded via [`add_ot_font`].
pub fn free_ot_fonts() -> Result<()> {
    // Safety: plain call.
    check(unsafe { sys::fbink_free_ot_fonts() })
}
