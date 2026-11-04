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
///
/// With the `serde` feature, a `Config` serializes as a plain map of the same
/// names as the builder methods (`{"row": 2, "font": "Unscii", ...}`), with
/// enums as variant-name strings. Missing fields deserialize as
/// [`Config::new`] defaults, so `{}` is a valid config.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "serde_impl::ConfigDef", into = "serde_impl::ConfigDef")
)]
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

/// Serde support: [`Config`] is a thin wrapper over the raw C struct (whose
/// fields are untyped integers), so it serializes through this friendly
/// mirror instead: one field per builder method, named after it, with enums
/// as variant names. Wired up via `serde(from = ..., into = ...)`.
#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Serde mirror of [`Config`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(default)]
    pub(crate) struct ConfigDef {
        pub row: i16,
        pub col: i16,
        pub hoffset: i16,
        pub voffset: i16,
        pub halfway: bool,
        pub fontmult: u8,
        pub font: Font,
        pub centered: bool,
        pub padded: bool,
        pub right_padded: bool,
        pub fg_color: FgColor,
        pub bg_color: BgColor,
        pub inverted: bool,
        pub overlay: bool,
        pub bgless: bool,
        pub fgless: bool,
        pub no_viewport: bool,
        pub flashing: bool,
        pub cleared: bool,
        pub waveform: WaveformMode,
        pub dithering: HwDither,
        pub sw_dithering: bool,
        pub cfa_mode: CfaMode,
        pub nightmode: bool,
        pub no_refresh: bool,
        pub no_merge: bool,
        pub animated: bool,
        pub halign: Align,
        pub valign: Align,
        pub scaled_width: i16,
        pub scaled_height: i16,
        pub ignore_alpha: bool,
        pub saturation_boost: u8,
        pub verbose: bool,
        pub quiet: bool,
        pub to_syslog: bool,
    }

    impl Default for ConfigDef {
        fn default() -> Self {
            // FBInk's documented defaults: a fully zeroed configuration.
            Config::new().into()
        }
    }

    impl From<Config> for ConfigDef {
        fn from(cfg: Config) -> Self {
            let raw = cfg.raw();
            Self {
                row: raw.row,
                col: raw.col,
                hoffset: raw.hoffset,
                voffset: raw.voffset,
                halfway: raw.is_halfway,
                fontmult: raw.fontmult,
                font: Font::from_raw(raw.fontname).unwrap_or(Font::Ibm),
                centered: raw.is_centered,
                padded: raw.is_padded,
                right_padded: raw.is_rpadded,
                fg_color: FgColor::from_raw(raw.fg_color).unwrap_or(FgColor::Black),
                bg_color: BgColor::from_raw(raw.bg_color).unwrap_or(BgColor::White),
                inverted: raw.is_inverted,
                overlay: raw.is_overlay,
                bgless: raw.is_bgless,
                fgless: raw.is_fgless,
                no_viewport: raw.no_viewport,
                flashing: raw.is_flashing,
                cleared: raw.is_cleared,
                waveform: WaveformMode::from_raw(raw.wfm_mode).unwrap_or(WaveformMode::Auto),
                dithering: HwDither::from_raw(raw.dithering_mode).unwrap_or(HwDither::Passthrough),
                sw_dithering: raw.sw_dithering,
                cfa_mode: CfaMode::from_raw(raw.cfa_mode).unwrap_or(CfaMode::Default),
                nightmode: raw.is_nightmode,
                no_refresh: raw.no_refresh,
                no_merge: raw.no_merge,
                animated: raw.is_animated,
                halign: Align::from_raw(raw.halign).unwrap_or(Align::None),
                valign: Align::from_raw(raw.valign).unwrap_or(Align::None),
                scaled_width: raw.scaled_width,
                scaled_height: raw.scaled_height,
                ignore_alpha: raw.ignore_alpha,
                saturation_boost: raw.saturation_boost,
                verbose: raw.is_verbose,
                quiet: raw.is_quiet,
                to_syslog: raw.to_syslog,
            }
        }
    }

    impl From<ConfigDef> for Config {
        fn from(def: ConfigDef) -> Self {
            Config::new()
                .row(def.row)
                .col(def.col)
                .hoffset(def.hoffset)
                .voffset(def.voffset)
                .halfway(def.halfway)
                .fontmult(def.fontmult)
                .font(def.font)
                .centered(def.centered)
                .padded(def.padded)
                .right_padded(def.right_padded)
                .fg_color(def.fg_color)
                .bg_color(def.bg_color)
                .inverted(def.inverted)
                .overlay(def.overlay)
                .bgless(def.bgless)
                .fgless(def.fgless)
                .no_viewport(def.no_viewport)
                .flashing(def.flashing)
                .cleared(def.cleared)
                .waveform(def.waveform)
                .dithering(def.dithering)
                .sw_dithering(def.sw_dithering)
                .cfa_mode(def.cfa_mode)
                .nightmode(def.nightmode)
                .no_refresh(def.no_refresh)
                .no_merge(def.no_merge)
                .animated(def.animated)
                .halign(def.halign)
                .valign(def.valign)
                .scaled_width(def.scaled_width)
                .scaled_height(def.scaled_height)
                .ignore_alpha(def.ignore_alpha)
                .saturation_boost(def.saturation_boost)
                .verbose(def.verbose)
                .quiet(def.quiet)
                .to_syslog(def.to_syslog)
        }
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

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrips_every_field() {
        let cfg = Config::new()
            .row(2)
            .col(-1)
            .hoffset(3)
            .voffset(-4)
            .halfway(true)
            .fontmult(2)
            .font(Font::UnsciiTall)
            .centered(true)
            .padded(true)
            .right_padded(true)
            .fg_color(FgColor::Gray7)
            .bg_color(BgColor::Gray3)
            .inverted(true)
            .overlay(true)
            .bgless(true)
            .fgless(true)
            .no_viewport(true)
            .flashing(true)
            .cleared(true)
            .waveform(WaveformMode::Gc16)
            .dithering(HwDither::Ordered)
            .sw_dithering(true)
            .cfa_mode(CfaMode::NtxSf)
            .nightmode(true)
            .no_refresh(true)
            .no_merge(true)
            .animated(true)
            .halign(Align::Center)
            .valign(Align::Edge)
            .scaled(100, 50)
            .ignore_alpha(true)
            .saturation_boost(30)
            .verbose(true)
            .quiet(true)
            .to_syslog(true);
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_impl::ConfigDef::from(back),
            serde_impl::ConfigDef::from(cfg)
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_uses_builder_names_and_variant_names() {
        let json = serde_json::to_string(
            &Config::new()
                .row(2)
                .font(Font::Unscii)
                .waveform(WaveformMode::Gc16),
        )
        .unwrap();
        assert!(json.contains("\"row\":2"));
        assert!(json.contains("\"font\":\"Unscii\""));
        assert!(json.contains("\"waveform\":\"Gc16\""));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_tolerates_missing_fields() {
        // An empty (or partial) config deserializes to FBInk's defaults.
        let cfg: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(
            serde_impl::ConfigDef::from(cfg),
            serde_impl::ConfigDef::default()
        );
        let cfg: Config = serde_json::from_str("{\"row\":3,\"font\":\"Cozette\"}").unwrap();
        assert_eq!(cfg.raw().row, 3);
        assert_eq!(cfg.raw().fontname, Font::Cozette.to_raw());
        assert_eq!(cfg.raw().wfm_mode, 0);
    }
}
