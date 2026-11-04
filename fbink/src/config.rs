//! The [`Config`] builder wrapping FBInk's `FBInkConfig` struct.
//!
//! A fully zeroed `FBInkConfig` is "perfectly sane" per `fbink.h`, so
//! [`Config::new`] gives you FBInk's defaults (black on white, `Auto`
//! waveform, ...), and every builder method only overrides what you ask for:
//!
//! ```
//! use fbink::{Config, Font};
//!
//! let cfg = Config::new().row(2).font(Font::Unscii).centered(true);
//! assert_eq!(cfg.raw().row, 2);
//! ```
//!
//! NOTE: a few fields are only honored by `fbink_init` (and thus "stick" until
//! the next init): `is_centered`, `fontmult`, `fontname`, `fg_color`,
//! `bg_color`, `no_viewport`, `is_verbose`, `is_quiet` & `to_syslog`.
//! Cheap updates are available via [`crate::update_verbosity`] &
//! [`crate::update_pen_colors`].

use crate::enums::{Align, BgColor, CfaMode, FgColor, Font, HwDither, WaveformMode};
use fbink_sys as sys;

/// An FBInk configuration, built fluently from FBInk's zeroed defaults.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    raw: sys::FBInkConfig,
}

impl Config {
    /// A zeroed configuration: exactly FBInk's documented defaults.
    pub fn new() -> Self {
        // FBInkConfig is a plain C struct of scalars: all-zero is valid and
        // is what FBInk itself expects as its "sane default".
        Self {
            raw: unsafe { std::mem::zeroed() },
        }
    }

    /// Wrap a raw `fbink-sys` configuration.
    pub fn from_raw(raw: sys::FBInkConfig) -> Self {
        Self { raw }
    }

    /// The raw `fbink-sys` configuration.
    pub fn raw(&self) -> &sys::FBInkConfig {
        &self.raw
    }

    /// The raw `fbink-sys` configuration, for direct field tweaks.
    pub fn raw_mut(&mut self) -> &mut sys::FBInkConfig {
        &mut self.raw
    }

    /// Consume the wrapper, yielding the raw configuration.
    pub fn into_raw(self) -> sys::FBInkConfig {
        self.raw
    }

    // Positioning (grid).

    /// Row (i.e. line) to print to; negative counts down from the bottom.
    pub fn row(mut self, row: i16) -> Self {
        self.raw.row = row;
        self
    }

    /// Column to print to; negative counts back from the right edge.
    pub fn col(mut self, col: i16) -> Self {
        self.raw.col = col;
        self
    }

    /// Horizontal offset (in pixels) applied to the text position.
    pub fn hoffset(mut self, px: i16) -> Self {
        self.raw.hoffset = px;
        self
    }

    /// Vertical offset (in pixels) applied to the text position.
    pub fn voffset(mut self, px: i16) -> Self {
        self.raw.voffset = px;
        self
    }

    /// Vertically center the text, honoring row offsets.
    pub fn halfway(mut self, yes: bool) -> Self {
        self.raw.is_halfway = yes;
        self
    }

    // Text rendering.

    /// Font scaling multiplier (e.g. `4` for x4); `0` means automatic.
    pub fn fontmult(mut self, mult: u8) -> Self {
        self.raw.fontmult = mult;
        self
    }

    /// Request a specific bundled fixed-cell font.
    pub fn font(mut self, font: Font) -> Self {
        self.raw.fontname = font.to_raw();
        self
    }

    /// Center the text horizontally.
    pub fn centered(mut self, yes: bool) -> Self {
        self.raw.is_centered = yes;
        self
    }

    /// Pad the text with blanks (on the left, or both sides if centered).
    pub fn padded(mut self, yes: bool) -> Self {
        self.raw.is_padded = yes;
        self
    }

    /// Right-pad the text with blanks.
    pub fn right_padded(mut self, yes: bool) -> Self {
        self.raw.is_rpadded = yes;
        self
    }

    /// Foreground pen color (eInk palette index).
    pub fn fg_color(mut self, color: FgColor) -> Self {
        self.raw.fg_color = color.to_raw();
        self
    }

    /// Background pen color (eInk palette index).
    pub fn bg_color(mut self, color: BgColor) -> Self {
        self.raw.bg_color = color.to_raw();
        self
    }

    /// Invert colors when *drawing* (text, clear & images).
    ///
    /// Not mutually exclusive with [`Config::nightmode`], and always supported.
    pub fn inverted(mut self, yes: bool) -> Self {
        self.raw.is_inverted = yes;
        self
    }

    /// Don't draw the background; the pen uses the inverse of the underlying
    /// pixel as its color.
    pub fn overlay(mut self, yes: bool) -> Self {
        self.raw.is_overlay = yes;
        self
    }

    /// Don't draw the background (mutually exclusive with
    /// [`Config::overlay`], which takes precedence).
    pub fn bgless(mut self, yes: bool) -> Self {
        self.raw.is_bgless = yes;
        self
    }

    /// Don't draw the foreground (takes precedence over
    /// [`Config::overlay`]/[`Config::bgless`]).
    pub fn fgless(mut self, yes: bool) -> Self {
        self.raw.is_fgless = yes;
        self
    }

    /// Ignore viewport corrections (hardware-related on Kobo, or row centering).
    pub fn no_viewport(mut self, yes: bool) -> Self {
        self.raw.no_viewport = yes;
        self
    }

    // Refresh behavior.

    /// Request a black flash on refresh (i.e. `UPDATE_MODE_FULL`).
    pub fn flashing(mut self, yes: bool) -> Self {
        self.raw.is_flashing = yes;
        self
    }

    /// Clear the full screen beforehand (honors `bg_color` & `inverted`).
    pub fn cleared(mut self, yes: bool) -> Self {
        self.raw.is_cleared = yes;
        self
    }

    /// Request a specific waveform mode.
    pub fn waveform(mut self, mode: WaveformMode) -> Self {
        self.raw.wfm_mode = mode.to_raw();
        self
    }

    /// Request a specific hardware dithering mode.
    pub fn dithering(mut self, mode: HwDither) -> Self {
        self.raw.dithering_mode = mode.to_raw();
        self
    }

    /// Request (ordered) *software* dithering when printing an image
    /// (not mutually exclusive with [`Config::dithering`]).
    pub fn sw_dithering(mut self, yes: bool) -> Self {
        self.raw.sw_dithering = yes;
        self
    }

    /// Request a specific CFA post-process mode (color panels, relevant
    /// waveform modes only).
    pub fn cfa_mode(mut self, mode: CfaMode) -> Self {
        self.raw.cfa_mode = mode.to_raw();
        self
    }

    /// Request hardware inversion (night mode), when supported/safe.
    /// Not mutually exclusive with [`Config::inverted`].
    pub fn nightmode(mut self, yes: bool) -> Self {
        self.raw.is_nightmode = yes;
        self
    }

    /// Skip actually refreshing the eInk screen (useful when drawing in batches).
    pub fn no_refresh(mut self, yes: bool) -> Self {
        self.raw.no_refresh = yes;
        self
    }

    /// Set the `EINK_NO_MERGE` flag (Kobo sunxi only).
    pub fn no_merge(mut self, yes: bool) -> Self {
        self.raw.no_merge = yes;
        self
    }

    /// Enable refresh animation, following [`crate::mtk_set_swipe_data`]
    /// (Kindle MTK only).
    pub fn animated(mut self, yes: bool) -> Self {
        self.raw.is_animated = yes;
        self
    }

    // Images & dumps.

    /// Horizontal alignment of images/dumps.
    pub fn halign(mut self, align: Align) -> Self {
        self.raw.halign = align.to_raw();
        self
    }

    /// Vertical alignment of images/dumps.
    pub fn valign(mut self, align: Align) -> Self {
        self.raw.valign = align.to_raw();
        self
    }

    /// Output width of images/dumps (`0` for no scaling, `-1` for viewport
    /// width).
    pub fn scaled_width(mut self, width: i16) -> Self {
        self.raw.scaled_width = width;
        self
    }

    /// Output height of images/dumps (`0` for no scaling, `-1` for viewport
    /// height).
    pub fn scaled_height(mut self, height: i16) -> Self {
        self.raw.scaled_height = height;
        self
    }

    /// Set both [`Config::scaled_width`] & [`Config::scaled_height`] at once.
    pub fn scaled(mut self, width: i16, height: i16) -> Self {
        self.raw.scaled_width = width;
        self.raw.scaled_height = height;
        self
    }

    /// Ignore any alpha channel in the source image (i.e. flatten it).
    pub fn ignore_alpha(mut self, yes: bool) -> Self {
        self.raw.ignore_alpha = yes;
        self
    }

    /// Boost image saturation, in % (useful on Kaleido panels; 32bpp only).
    pub fn saturation_boost(mut self, percent: u8) -> Self {
        self.raw.saturation_boost = percent;
        self
    }

    // Diagnostics.

    /// Print verbose diagnostic information on stdout.
    pub fn verbose(mut self, yes: bool) -> Self {
        self.raw.is_verbose = yes;
        self
    }

    /// Hide `fbink_init`'s hardware setup info (sent to stderr).
    pub fn quiet(mut self, yes: bool) -> Self {
        self.raw.is_quiet = yes;
        self
    }

    /// Send messages & errors to the syslog instead of stdout/stderr.
    pub fn to_syslog(mut self, yes: bool) -> Self {
        self.raw.to_syslog = yes;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zeroed() {
        let cfg = Config::new();
        assert_eq!(cfg.raw().row, 0);
        assert_eq!(cfg.raw().fontname, 0);
        assert!(!cfg.raw().is_centered);
        assert_eq!(cfg.raw().wfm_mode, 0);
        assert_eq!(cfg.raw().saturation_boost, 0);
    }

    #[test]
    fn builders_only_touch_their_field() {
        let cfg = Config::new()
            .row(2)
            .col(-1)
            .font(Font::Unscii)
            .centered(true)
            .fg_color(FgColor::Gray7)
            .bg_color(BgColor::Gray3)
            .waveform(WaveformMode::Gc16)
            .dithering(HwDither::Ordered)
            .nightmode(true)
            .saturation_boost(50);
        assert_eq!(cfg.raw().row, 2);
        assert_eq!(cfg.raw().col, -1);
        assert_eq!(cfg.raw().fontname, Font::Unscii.to_raw());
        assert!(cfg.raw().is_centered);
        assert_eq!(cfg.raw().fg_color, FgColor::Gray7.to_raw());
        assert_eq!(cfg.raw().bg_color, BgColor::Gray3.to_raw());
        assert_eq!(cfg.raw().wfm_mode, WaveformMode::Gc16.to_raw());
        assert_eq!(cfg.raw().dithering_mode, HwDither::Ordered.to_raw());
        assert!(cfg.raw().is_nightmode);
        assert_eq!(cfg.raw().saturation_boost, 50);
        // Untouched fields stay at their defaults.
        assert!(!cfg.raw().is_flashing);
        assert_eq!(cfg.raw().halign, 0);
    }
}
