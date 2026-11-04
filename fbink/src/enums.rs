//! Safe, idiomatic wrappers around FBInk's packed C enums.
//!
//! Every conversion to/from the raw `fbink-sys` values goes through the
//! `sys::*` constants generated from `fbink.h` (values are never hardcoded
//! here). Note that FBInk typedefs its packed enums to fixed-width integers:
//!
//! * [`SunxiForceRota`] is `i8` (`SUNXI_FORCE_ROTA_INDEX_T`),
//! * [`FontStyle`] is `c_int` (`FONT_STYLE_T`, not packed, for ABI reasons),
//! * [`MtkHalftone`] is `i32` (`MTK_HALFTONE_MODE_INDEX_T`),
//! * everything else is `u8` (e.g. `FONT_INDEX_T`).
//!
//! # String representations
//!
//! Every fieldless enum also implements `Display` & `FromStr` (via `strum`),
//! round-tripping its Rust variant names (`"Gc16"`, `"UnsciiAlt"`, ...), plus
//! `EnumIter`/`EnumCount` for iteration (e.g. over every [`WaveformMode`]).
//! With the `serde` feature, all enums serialize the same way (variant names
//! as strings; data-carrying variants use serde's externally-tagged form,
//! e.g. `{"Custom": 3}` for [`MtkHalftone::Custom`]).

use fbink_sys as sys;

/// Bundled fixed-cell font (`FONT_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Font {
    Ibm,
    Unscii,
    UnsciiAlt,
    UnsciiThin,
    UnsciiFantasy,
    UnsciiMcr,
    UnsciiTall,
    Block,
    Leggie,
    Veggie,
    Kates,
    Fkp,
    Ctrld,
    Orp,
    Orpb,
    Orpi,
    Scientifica,
    ScientificaB,
    ScientificaI,
    Terminus,
    TerminusB,
    Fatty,
    Spleen,
    Tewi,
    TewiB,
    Topaz,
    MicroKnight,
    Vga,
    Unifont,
    UnifontDw,
    Cozette,
}

impl Font {
    pub const fn to_raw(self) -> sys::FONT_INDEX_T {
        use sys::FONT_INDEX_E as E;
        match self {
            Font::Ibm => E::IBM,
            Font::Unscii => E::UNSCII,
            Font::UnsciiAlt => E::UNSCII_ALT,
            Font::UnsciiThin => E::UNSCII_THIN,
            Font::UnsciiFantasy => E::UNSCII_FANTASY,
            Font::UnsciiMcr => E::UNSCII_MCR,
            Font::UnsciiTall => E::UNSCII_TALL,
            Font::Block => E::BLOCK,
            Font::Leggie => E::LEGGIE,
            Font::Veggie => E::VEGGIE,
            Font::Kates => E::KATES,
            Font::Fkp => E::FKP,
            Font::Ctrld => E::CTRLD,
            Font::Orp => E::ORP,
            Font::Orpb => E::ORPB,
            Font::Orpi => E::ORPI,
            Font::Scientifica => E::SCIENTIFICA,
            Font::ScientificaB => E::SCIENTIFICAB,
            Font::ScientificaI => E::SCIENTIFICAI,
            Font::Terminus => E::TERMINUS,
            Font::TerminusB => E::TERMINUSB,
            Font::Fatty => E::FATTY,
            Font::Spleen => E::SPLEEN,
            Font::Tewi => E::TEWI,
            Font::TewiB => E::TEWIB,
            Font::Topaz => E::TOPAZ,
            Font::MicroKnight => E::MICROKNIGHT,
            Font::Vga => E::VGA,
            Font::Unifont => E::UNIFONT,
            Font::UnifontDw => E::UNIFONTDW,
            Font::Cozette => E::COZETTE,
        }
    }

    pub const fn from_raw(v: sys::FONT_INDEX_T) -> Option<Font> {
        use sys::FONT_INDEX_E as E;
        match v {
            E::IBM => Some(Font::Ibm),
            E::UNSCII => Some(Font::Unscii),
            E::UNSCII_ALT => Some(Font::UnsciiAlt),
            E::UNSCII_THIN => Some(Font::UnsciiThin),
            E::UNSCII_FANTASY => Some(Font::UnsciiFantasy),
            E::UNSCII_MCR => Some(Font::UnsciiMcr),
            E::UNSCII_TALL => Some(Font::UnsciiTall),
            E::BLOCK => Some(Font::Block),
            E::LEGGIE => Some(Font::Leggie),
            E::VEGGIE => Some(Font::Veggie),
            E::KATES => Some(Font::Kates),
            E::FKP => Some(Font::Fkp),
            E::CTRLD => Some(Font::Ctrld),
            E::ORP => Some(Font::Orp),
            E::ORPB => Some(Font::Orpb),
            E::ORPI => Some(Font::Orpi),
            E::SCIENTIFICA => Some(Font::Scientifica),
            E::SCIENTIFICAB => Some(Font::ScientificaB),
            E::SCIENTIFICAI => Some(Font::ScientificaI),
            E::TERMINUS => Some(Font::Terminus),
            E::TERMINUSB => Some(Font::TerminusB),
            E::FATTY => Some(Font::Fatty),
            E::SPLEEN => Some(Font::Spleen),
            E::TEWI => Some(Font::Tewi),
            E::TEWIB => Some(Font::TewiB),
            E::TOPAZ => Some(Font::Topaz),
            E::MICROKNIGHT => Some(Font::MicroKnight),
            E::VGA => Some(Font::Vga),
            E::UNIFONT => Some(Font::Unifont),
            E::UNIFONTDW => Some(Font::UnifontDw),
            E::COZETTE => Some(Font::Cozette),
            _ => None,
        }
    }
}

/// OpenType font style (`FONT_STYLE_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum FontStyle {
    Regular,
    Italic,
    Bold,
    BoldItalic,
}

impl FontStyle {
    pub const fn to_raw(self) -> sys::FONT_STYLE_T {
        use sys::FONT_STYLE_E as E;
        match self {
            FontStyle::Regular => E::FNT_REGULAR as sys::FONT_STYLE_T,
            FontStyle::Italic => E::FNT_ITALIC as sys::FONT_STYLE_T,
            FontStyle::Bold => E::FNT_BOLD as sys::FONT_STYLE_T,
            FontStyle::BoldItalic => E::FNT_BOLD_ITALIC as sys::FONT_STYLE_T,
        }
    }

    pub const fn from_raw(v: sys::FONT_STYLE_T) -> Option<FontStyle> {
        use sys::FONT_STYLE_E as E;
        match v {
            x if x == E::FNT_REGULAR as sys::FONT_STYLE_T => Some(FontStyle::Regular),
            x if x == E::FNT_ITALIC as sys::FONT_STYLE_T => Some(FontStyle::Italic),
            x if x == E::FNT_BOLD as sys::FONT_STYLE_T => Some(FontStyle::Bold),
            x if x == E::FNT_BOLD_ITALIC as sys::FONT_STYLE_T => Some(FontStyle::BoldItalic),
            _ => None,
        }
    }
}

/// Horizontal/vertical alignment (`ALIGN_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Align {
    /// Left (halign) or top (valign).
    None,
    Center,
    /// Right (halign) or bottom (valign).
    Edge,
}

impl Align {
    pub const fn to_raw(self) -> sys::ALIGN_INDEX_T {
        use sys::ALIGN_INDEX_E as E;
        match self {
            Align::None => E::NONE,
            Align::Center => E::CENTER,
            Align::Edge => E::EDGE,
        }
    }

    pub const fn from_raw(v: sys::ALIGN_INDEX_T) -> Option<Align> {
        use sys::ALIGN_INDEX_E as E;
        match v {
            E::NONE => Some(Align::None),
            E::CENTER => Some(Align::Center),
            E::EDGE => Some(Align::Edge),
            _ => None,
        }
    }
}

/// Drawing-area padding for OpenType rendering (`PADDING_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Padding {
    None,
    Horizontal,
    Vertical,
    Full,
}

impl Padding {
    pub const fn to_raw(self) -> sys::PADDING_INDEX_T {
        use sys::PADDING_INDEX_E as E;
        match self {
            Padding::None => E::NO_PADDING,
            Padding::Horizontal => E::HORI_PADDING,
            Padding::Vertical => E::VERT_PADDING,
            Padding::Full => E::FULL_PADDING,
        }
    }

    pub const fn from_raw(v: sys::PADDING_INDEX_T) -> Option<Padding> {
        use sys::PADDING_INDEX_E as E;
        match v {
            E::NO_PADDING => Some(Padding::None),
            E::HORI_PADDING => Some(Padding::Horizontal),
            E::VERT_PADDING => Some(Padding::Vertical),
            E::FULL_PADDING => Some(Padding::Full),
            _ => None,
        }
    }
}

/// Foreground pen color, as an eInk palette index (`FG_COLOR_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum FgColor {
    Black,
    Gray1,
    Gray2,
    Gray3,
    Gray4,
    Gray5,
    Gray6,
    Gray7,
    Gray8,
    Gray9,
    Gray10,
    Gray11,
    Gray12,
    Gray13,
    Gray14,
    White,
}

impl FgColor {
    pub const fn to_raw(self) -> sys::FG_COLOR_INDEX_T {
        use sys::FG_COLOR_INDEX_E as E;
        match self {
            FgColor::Black => E::FG_BLACK,
            FgColor::Gray1 => E::FG_GRAY1,
            FgColor::Gray2 => E::FG_GRAY2,
            FgColor::Gray3 => E::FG_GRAY3,
            FgColor::Gray4 => E::FG_GRAY4,
            FgColor::Gray5 => E::FG_GRAY5,
            FgColor::Gray6 => E::FG_GRAY6,
            FgColor::Gray7 => E::FG_GRAY7,
            FgColor::Gray8 => E::FG_GRAY8,
            FgColor::Gray9 => E::FG_GRAY9,
            FgColor::Gray10 => E::FG_GRAYA,
            FgColor::Gray11 => E::FG_GRAYB,
            FgColor::Gray12 => E::FG_GRAYC,
            FgColor::Gray13 => E::FG_GRAYD,
            FgColor::Gray14 => E::FG_GRAYE,
            FgColor::White => E::FG_WHITE,
        }
    }

    pub const fn from_raw(v: sys::FG_COLOR_INDEX_T) -> Option<FgColor> {
        use sys::FG_COLOR_INDEX_E as E;
        match v {
            E::FG_BLACK => Some(FgColor::Black),
            E::FG_GRAY1 => Some(FgColor::Gray1),
            E::FG_GRAY2 => Some(FgColor::Gray2),
            E::FG_GRAY3 => Some(FgColor::Gray3),
            E::FG_GRAY4 => Some(FgColor::Gray4),
            E::FG_GRAY5 => Some(FgColor::Gray5),
            E::FG_GRAY6 => Some(FgColor::Gray6),
            E::FG_GRAY7 => Some(FgColor::Gray7),
            E::FG_GRAY8 => Some(FgColor::Gray8),
            E::FG_GRAY9 => Some(FgColor::Gray9),
            E::FG_GRAYA => Some(FgColor::Gray10),
            E::FG_GRAYB => Some(FgColor::Gray11),
            E::FG_GRAYC => Some(FgColor::Gray12),
            E::FG_GRAYD => Some(FgColor::Gray13),
            E::FG_GRAYE => Some(FgColor::Gray14),
            E::FG_WHITE => Some(FgColor::White),
            _ => None,
        }
    }
}

/// Background pen color, as an eInk palette index (`BG_COLOR_INDEX_E`).
///
/// The C palette enumerates *downwards* (`BG_WHITE` is 0, `BG_GRAYE` is 1,
/// ...): the variant names here refer to the same gray *levels* as
/// [`FgColor`] (i.e. `Gray1` is `0x11` on both), whatever their raw index.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BgColor {
    White,
    Gray14,
    Gray13,
    Gray12,
    Gray11,
    Gray10,
    Gray9,
    Gray8,
    Gray7,
    Gray6,
    Gray5,
    Gray4,
    Gray3,
    Gray2,
    Gray1,
    Black,
}

impl BgColor {
    pub const fn to_raw(self) -> sys::BG_COLOR_INDEX_T {
        use sys::BG_COLOR_INDEX_E as E;
        match self {
            BgColor::White => E::BG_WHITE,
            BgColor::Gray1 => E::BG_GRAY1,
            BgColor::Gray2 => E::BG_GRAY2,
            BgColor::Gray3 => E::BG_GRAY3,
            BgColor::Gray4 => E::BG_GRAY4,
            BgColor::Gray5 => E::BG_GRAY5,
            BgColor::Gray6 => E::BG_GRAY6,
            BgColor::Gray7 => E::BG_GRAY7,
            BgColor::Gray8 => E::BG_GRAY8,
            BgColor::Gray9 => E::BG_GRAY9,
            BgColor::Gray10 => E::BG_GRAYA,
            BgColor::Gray11 => E::BG_GRAYB,
            BgColor::Gray12 => E::BG_GRAYC,
            BgColor::Gray13 => E::BG_GRAYD,
            BgColor::Gray14 => E::BG_GRAYE,
            BgColor::Black => E::BG_BLACK,
        }
    }

    pub const fn from_raw(v: sys::BG_COLOR_INDEX_T) -> Option<BgColor> {
        use sys::BG_COLOR_INDEX_E as E;
        match v {
            E::BG_WHITE => Some(BgColor::White),
            E::BG_GRAY1 => Some(BgColor::Gray1),
            E::BG_GRAY2 => Some(BgColor::Gray2),
            E::BG_GRAY3 => Some(BgColor::Gray3),
            E::BG_GRAY4 => Some(BgColor::Gray4),
            E::BG_GRAY5 => Some(BgColor::Gray5),
            E::BG_GRAY6 => Some(BgColor::Gray6),
            E::BG_GRAY7 => Some(BgColor::Gray7),
            E::BG_GRAY8 => Some(BgColor::Gray8),
            E::BG_GRAY9 => Some(BgColor::Gray9),
            E::BG_GRAYA => Some(BgColor::Gray10),
            E::BG_GRAYB => Some(BgColor::Gray11),
            E::BG_GRAYC => Some(BgColor::Gray12),
            E::BG_GRAYD => Some(BgColor::Gray13),
            E::BG_GRAYE => Some(BgColor::Gray14),
            E::BG_BLACK => Some(BgColor::Black),
            _ => None,
        }
    }
}

/// eInk waveform (refresh) mode (`WFM_MODE_INDEX_E`).
///
/// Which modes are *actually* available is highly device-dependent: c.f. the
/// notes in `fbink.h` (in doubt, [`WaveformMode::Auto`] is the safe default).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum WaveformMode {
    /// Let the EPDC choose via histogram analysis (the default).
    Auto,
    /// Fast B&W update, light ghosting, never flashes.
    Du,
    /// High fidelity (lowest ghosting), ideal for images.
    Gc16,
    /// 4-level gray.
    Gc4,
    /// Fast B&W animation mode, never flashes.
    A2,
    /// From white to any; typically optimized for text.
    Gl16,
    /// "Regal": from white to any with ghosting/flash reduction.
    Reagl,
    /// Like [`WaveformMode::Reagl`], more ghosting reduction.
    Reagld,
    /// Lower latency [`WaveformMode::Gc16`] (mostly Kindle).
    Gc16Fast,
    /// Lower latency [`WaveformMode::Gl16`] (mostly Kindle).
    Gl16Fast,
    /// 4-level gray, never flashes.
    Du4,
    /// From white to B/W/gray.
    Gl4,
    /// From *black* to any (text on black, e.g. nightmode).
    Gl16Inv,
    /// Nightmode GC16.
    Gck16,
    /// Nightmode GL16 variant (Kobo Mk. 9/11/12).
    Glkw16,
    /// Slow init mode (flashes to white); for documentation purposes.
    Init,
    /// Unknown mode.
    Unknown,
    /// reMarkable-only init mode.
    Init2,
    /// PocketBook-only A2 variant.
    A2In,
    /// PocketBook-only A2 variant.
    A2Out,
    /// i.MX-only; alias for REAGL (or REAGLD when flashing).
    Gc16Hq,
    /// B288-only (PocketBook).
    Gs16,
    /// Kobo sunxi-only GL16 variant.
    Gu16,
    /// Kobo sunxi-only nightmode GL16.
    Glk16,
    /// Kobo sunxi-only local GC16.
    Clear,
    /// Kobo sunxi-only local GC4.
    Gc4L,
    /// Color content on Kaleido panels (Kobo sunxi / MTK).
    Gcc16,
    /// Color highlights on text on Kaleido panels.
    Glrc16,
    /// Internal use (Kindle MTK).
    Gc16Partial,
    /// Internal use (Kindle MTK).
    Gck16Partial,
    /// DU for white-on-black (Kindle MTK).
    Dunm,
    /// Internal use (Kindle MTK swipe animation).
    P2Sw,
    /// Nightmode [`WaveformMode::Gcc16`] (Kindle MTK + CFA).
    Gcck16,
    /// Nightmode [`WaveformMode::Glrc16`] (Kindle MTK + CFA).
    Glrck16,
}

impl WaveformMode {
    pub const fn to_raw(self) -> sys::WFM_MODE_INDEX_T {
        use sys::WFM_MODE_INDEX_E as E;
        match self {
            WaveformMode::Auto => E::WFM_AUTO,
            WaveformMode::Du => E::WFM_DU,
            WaveformMode::Gc16 => E::WFM_GC16,
            WaveformMode::Gc4 => E::WFM_GC4,
            WaveformMode::A2 => E::WFM_A2,
            WaveformMode::Gl16 => E::WFM_GL16,
            WaveformMode::Reagl => E::WFM_REAGL,
            WaveformMode::Reagld => E::WFM_REAGLD,
            WaveformMode::Gc16Fast => E::WFM_GC16_FAST,
            WaveformMode::Gl16Fast => E::WFM_GL16_FAST,
            WaveformMode::Du4 => E::WFM_DU4,
            WaveformMode::Gl4 => E::WFM_GL4,
            WaveformMode::Gl16Inv => E::WFM_GL16_INV,
            WaveformMode::Gck16 => E::WFM_GCK16,
            WaveformMode::Glkw16 => E::WFM_GLKW16,
            WaveformMode::Init => E::WFM_INIT,
            WaveformMode::Unknown => E::WFM_UNKNOWN,
            WaveformMode::Init2 => E::WFM_INIT2,
            WaveformMode::A2In => E::WFM_A2IN,
            WaveformMode::A2Out => E::WFM_A2OUT,
            WaveformMode::Gc16Hq => E::WFM_GC16HQ,
            WaveformMode::Gs16 => E::WFM_GS16,
            WaveformMode::Gu16 => E::WFM_GU16,
            WaveformMode::Glk16 => E::WFM_GLK16,
            WaveformMode::Clear => E::WFM_CLEAR,
            WaveformMode::Gc4L => E::WFM_GC4L,
            WaveformMode::Gcc16 => E::WFM_GCC16,
            WaveformMode::Glrc16 => E::WFM_GLRC16,
            WaveformMode::Gc16Partial => E::WFM_GC16_PARTIAL,
            WaveformMode::Gck16Partial => E::WFM_GCK16_PARTIAL,
            WaveformMode::Dunm => E::WFM_DUNM,
            WaveformMode::P2Sw => E::WFM_P2SW,
            WaveformMode::Gcck16 => E::WFM_GCCK16,
            WaveformMode::Glrck16 => E::WFM_GLRCK16,
        }
    }

    pub const fn from_raw(v: sys::WFM_MODE_INDEX_T) -> Option<WaveformMode> {
        use sys::WFM_MODE_INDEX_E as E;
        match v {
            E::WFM_AUTO => Some(WaveformMode::Auto),
            E::WFM_DU => Some(WaveformMode::Du),
            E::WFM_GC16 => Some(WaveformMode::Gc16),
            E::WFM_GC4 => Some(WaveformMode::Gc4),
            E::WFM_A2 => Some(WaveformMode::A2),
            E::WFM_GL16 => Some(WaveformMode::Gl16),
            E::WFM_REAGL => Some(WaveformMode::Reagl),
            E::WFM_REAGLD => Some(WaveformMode::Reagld),
            E::WFM_GC16_FAST => Some(WaveformMode::Gc16Fast),
            E::WFM_GL16_FAST => Some(WaveformMode::Gl16Fast),
            E::WFM_DU4 => Some(WaveformMode::Du4),
            E::WFM_GL4 => Some(WaveformMode::Gl4),
            E::WFM_GL16_INV => Some(WaveformMode::Gl16Inv),
            E::WFM_GCK16 => Some(WaveformMode::Gck16),
            E::WFM_GLKW16 => Some(WaveformMode::Glkw16),
            E::WFM_INIT => Some(WaveformMode::Init),
            E::WFM_UNKNOWN => Some(WaveformMode::Unknown),
            E::WFM_INIT2 => Some(WaveformMode::Init2),
            E::WFM_A2IN => Some(WaveformMode::A2In),
            E::WFM_A2OUT => Some(WaveformMode::A2Out),
            E::WFM_GC16HQ => Some(WaveformMode::Gc16Hq),
            E::WFM_GS16 => Some(WaveformMode::Gs16),
            E::WFM_GU16 => Some(WaveformMode::Gu16),
            E::WFM_GLK16 => Some(WaveformMode::Glk16),
            E::WFM_CLEAR => Some(WaveformMode::Clear),
            E::WFM_GC4L => Some(WaveformMode::Gc4L),
            E::WFM_GCC16 => Some(WaveformMode::Gcc16),
            E::WFM_GLRC16 => Some(WaveformMode::Glrc16),
            E::WFM_GC16_PARTIAL => Some(WaveformMode::Gc16Partial),
            E::WFM_GCK16_PARTIAL => Some(WaveformMode::Gck16Partial),
            E::WFM_DUNM => Some(WaveformMode::Dunm),
            E::WFM_P2SW => Some(WaveformMode::P2Sw),
            E::WFM_GCCK16 => Some(WaveformMode::Gcck16),
            E::WFM_GLRCK16 => Some(WaveformMode::Glrck16),
            _ => None,
        }
    }
}

/// Hardware dithering mode (`HW_DITHER_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum HwDither {
    /// No dithering (the default).
    Passthrough,
    FloydSteinberg,
    Atkinson,
    /// Generally the only supported HW variant on EPDC v2.
    Ordered,
    QuantOnly,
    /// Legacy (in-kernel, software) EPDC v1 dithering.
    Legacy,
}

impl HwDither {
    pub const fn to_raw(self) -> sys::HW_DITHER_INDEX_T {
        use sys::HW_DITHER_INDEX_E as E;
        match self {
            HwDither::Passthrough => E::HWD_PASSTHROUGH,
            HwDither::FloydSteinberg => E::HWD_FLOYD_STEINBERG,
            HwDither::Atkinson => E::HWD_ATKINSON,
            HwDither::Ordered => E::HWD_ORDERED,
            HwDither::QuantOnly => E::HWD_QUANT_ONLY,
            HwDither::Legacy => E::HWD_LEGACY,
        }
    }

    pub const fn from_raw(v: sys::HW_DITHER_INDEX_T) -> Option<HwDither> {
        use sys::HW_DITHER_INDEX_E as E;
        match v {
            E::HWD_PASSTHROUGH => Some(HwDither::Passthrough),
            E::HWD_FLOYD_STEINBERG => Some(HwDither::FloydSteinberg),
            E::HWD_ATKINSON => Some(HwDither::Atkinson),
            E::HWD_ORDERED => Some(HwDither::Ordered),
            E::HWD_QUANT_ONLY => Some(HwDither::QuantOnly),
            E::HWD_LEGACY => Some(HwDither::Legacy),
            _ => None,
        }
    }
}

/// CFA (color filter array) post-process mode (`CFA_MODE_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CfaMode {
    Default,
    AieS4,
    AieS7,
    AieS9,
    G0,
    G1,
    G2,
    Ntx,
    NtxSf,
    Skip,
}

impl CfaMode {
    pub const fn to_raw(self) -> sys::CFA_MODE_INDEX_T {
        use sys::CFA_MODE_INDEX_E as E;
        match self {
            CfaMode::Default => E::CFA_DEFAULT,
            CfaMode::AieS4 => E::CFA_AIE_S4,
            CfaMode::AieS7 => E::CFA_AIE_S7,
            CfaMode::AieS9 => E::CFA_AIE_S9,
            CfaMode::G0 => E::CFA_G0,
            CfaMode::G1 => E::CFA_G1,
            CfaMode::G2 => E::CFA_G2,
            CfaMode::Ntx => E::CFA_NTX,
            CfaMode::NtxSf => E::CFA_NTX_SF,
            CfaMode::Skip => E::CFA_SKIP,
        }
    }

    pub const fn from_raw(v: sys::CFA_MODE_INDEX_T) -> Option<CfaMode> {
        use sys::CFA_MODE_INDEX_E as E;
        match v {
            E::CFA_DEFAULT => Some(CfaMode::Default),
            E::CFA_AIE_S4 => Some(CfaMode::AieS4),
            E::CFA_AIE_S7 => Some(CfaMode::AieS7),
            E::CFA_AIE_S9 => Some(CfaMode::AieS9),
            E::CFA_G0 => Some(CfaMode::G0),
            E::CFA_G1 => Some(CfaMode::G1),
            E::CFA_G2 => Some(CfaMode::G2),
            E::CFA_NTX => Some(CfaMode::Ntx),
            E::CFA_NTX_SF => Some(CfaMode::NtxSf),
            E::CFA_SKIP => Some(CfaMode::Skip),
            _ => None,
        }
    }
}

/// NTX (Kobo) kernel rotation quirk type (`NTX_ROTA_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NtxRotaQuirk {
    /// No kernel rotation shenanigans.
    Straight,
    /// Every rotation is inverted by the kernel.
    AllInverted,
    /// Only landscape (odd) rotations are inverted by the kernel.
    OddInverted,
    /// Deprecated: [`NtxRotaQuirk::Straight`] + boot rota is portrait.
    Sane,
    /// sunxi: the rotate flag is technically meaningless.
    Sunxi,
    /// No kernel shenanigans, touch panel mounted inverted.
    CwTouch,
    /// No kernel shenanigans, touch panel mounted as usual.
    CcwTouch,
}

impl NtxRotaQuirk {
    pub const fn to_raw(self) -> sys::NTX_ROTA_INDEX_T {
        use sys::NTX_ROTA_INDEX_E as E;
        match self {
            NtxRotaQuirk::Straight => E::NTX_ROTA_STRAIGHT,
            NtxRotaQuirk::AllInverted => E::NTX_ROTA_ALL_INVERTED,
            NtxRotaQuirk::OddInverted => E::NTX_ROTA_ODD_INVERTED,
            NtxRotaQuirk::Sane => E::NTX_ROTA_SANE,
            NtxRotaQuirk::Sunxi => E::NTX_ROTA_SUNXI,
            NtxRotaQuirk::CwTouch => E::NTX_ROTA_CW_TOUCH,
            NtxRotaQuirk::CcwTouch => E::NTX_ROTA_CCW_TOUCH,
        }
    }

    pub const fn from_raw(v: sys::NTX_ROTA_INDEX_T) -> Option<NtxRotaQuirk> {
        use sys::NTX_ROTA_INDEX_E as E;
        match v {
            E::NTX_ROTA_STRAIGHT => Some(NtxRotaQuirk::Straight),
            E::NTX_ROTA_ALL_INVERTED => Some(NtxRotaQuirk::AllInverted),
            E::NTX_ROTA_ODD_INVERTED => Some(NtxRotaQuirk::OddInverted),
            E::NTX_ROTA_SANE => Some(NtxRotaQuirk::Sane),
            E::NTX_ROTA_SUNXI => Some(NtxRotaQuirk::Sunxi),
            E::NTX_ROTA_CW_TOUCH => Some(NtxRotaQuirk::CwTouch),
            E::NTX_ROTA_CCW_TOUCH => Some(NtxRotaQuirk::CcwTouch),
            _ => None,
        }
    }
}

/// Rotation policy for sunxi (Kobo Elipsa/Sage) devices
/// (`SUNXI_FORCE_ROTA_INDEX_E`; raw type is `i8`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum SunxiForceRota {
    /// Not supported (i.e. not a sunxi device).
    NotSup,
    /// Honor the gyro if it matches the working buffer's rotation (needs fbdamage).
    CurrentRota,
    /// Honor the gyro if it matches the working buffer's layout (needs fbdamage).
    CurrentLayout,
    /// Honor the gyro if it matches a portrait layout.
    Portrait,
    /// Honor the gyro if it matches a landscape layout.
    Landscape,
    /// Honor the gyro (the default).
    Gyro,
    /// `FB_ROTATE_UR`.
    Ur,
    /// `FB_ROTATE_CW`.
    Cw,
    /// `FB_ROTATE_UD`.
    Ud,
    /// `FB_ROTATE_CCW`.
    Ccw,
    /// Match the working buffer's rotation (needs fbdamage).
    WorkBuf,
}

impl SunxiForceRota {
    pub const fn to_raw(self) -> sys::SUNXI_FORCE_ROTA_INDEX_T {
        use sys::SUNXI_FORCE_ROTA_INDEX_E as E;
        match self {
            SunxiForceRota::NotSup => E::FORCE_ROTA_NOTSUP,
            SunxiForceRota::CurrentRota => E::FORCE_ROTA_CURRENT_ROTA,
            SunxiForceRota::CurrentLayout => E::FORCE_ROTA_CURRENT_LAYOUT,
            SunxiForceRota::Portrait => E::FORCE_ROTA_PORTRAIT,
            SunxiForceRota::Landscape => E::FORCE_ROTA_LANDSCAPE,
            SunxiForceRota::Gyro => E::FORCE_ROTA_GYRO,
            SunxiForceRota::Ur => E::FORCE_ROTA_UR,
            SunxiForceRota::Cw => E::FORCE_ROTA_CW,
            SunxiForceRota::Ud => E::FORCE_ROTA_UD,
            SunxiForceRota::Ccw => E::FORCE_ROTA_CCW,
            SunxiForceRota::WorkBuf => E::FORCE_ROTA_WORKBUF,
        }
    }

    pub const fn from_raw(v: sys::SUNXI_FORCE_ROTA_INDEX_T) -> Option<SunxiForceRota> {
        use sys::SUNXI_FORCE_ROTA_INDEX_E as E;
        match v {
            E::FORCE_ROTA_NOTSUP => Some(SunxiForceRota::NotSup),
            E::FORCE_ROTA_CURRENT_ROTA => Some(SunxiForceRota::CurrentRota),
            E::FORCE_ROTA_CURRENT_LAYOUT => Some(SunxiForceRota::CurrentLayout),
            E::FORCE_ROTA_PORTRAIT => Some(SunxiForceRota::Portrait),
            E::FORCE_ROTA_LANDSCAPE => Some(SunxiForceRota::Landscape),
            E::FORCE_ROTA_GYRO => Some(SunxiForceRota::Gyro),
            E::FORCE_ROTA_UR => Some(SunxiForceRota::Ur),
            E::FORCE_ROTA_CW => Some(SunxiForceRota::Cw),
            E::FORCE_ROTA_UD => Some(SunxiForceRota::Ud),
            E::FORCE_ROTA_CCW => Some(SunxiForceRota::Ccw),
            E::FORCE_ROTA_WORKBUF => Some(SunxiForceRota::WorkBuf),
            _ => None,
        }
    }
}

/// Swipe direction for the Kindle MTK refresh animation
/// (`MTK_SWIPE_DIRECTION_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MtkSwipeDirection {
    Down,
    Up,
    Left,
    Right,
}

impl MtkSwipeDirection {
    pub const fn to_raw(self) -> sys::MTK_SWIPE_DIRECTION_INDEX_T {
        use sys::MTK_SWIPE_DIRECTION_INDEX_E as E;
        match self {
            MtkSwipeDirection::Down => E::MTK_SWIPE_DIR_DOWN,
            MtkSwipeDirection::Up => E::MTK_SWIPE_DIR_UP,
            MtkSwipeDirection::Left => E::MTK_SWIPE_DIR_LEFT,
            MtkSwipeDirection::Right => E::MTK_SWIPE_DIR_RIGHT,
        }
    }

    pub const fn from_raw(v: sys::MTK_SWIPE_DIRECTION_INDEX_T) -> Option<MtkSwipeDirection> {
        use sys::MTK_SWIPE_DIRECTION_INDEX_E as E;
        match v {
            E::MTK_SWIPE_DIR_DOWN => Some(MtkSwipeDirection::Down),
            E::MTK_SWIPE_DIR_UP => Some(MtkSwipeDirection::Up),
            E::MTK_SWIPE_DIR_LEFT => Some(MtkSwipeDirection::Left),
            E::MTK_SWIPE_DIR_RIGHT => Some(MtkSwipeDirection::Right),
            _ => None,
        }
    }
}

/// Halftone (checkered pattern) mode for Kindle MTK
/// (`MTK_HALFTONE_MODE_INDEX_E`; raw type is `i32`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MtkHalftone {
    /// Feature disabled.
    Disabled,
    /// Default checker size (1).
    Default,
    /// Custom raw size value: the *actual* checker size is `size - 1` pixels.
    ///
    /// Values `0` and `1` are indistinguishable from [`MtkHalftone::Disabled`]
    /// & [`MtkHalftone::Default`] (and convert back to them): use `>= 2`.
    Custom(i32),
}

impl MtkHalftone {
    pub const fn to_raw(self) -> sys::MTK_HALFTONE_MODE_INDEX_T {
        use sys::MTK_HALFTONE_MODE_INDEX_E as E;
        match self {
            MtkHalftone::Disabled => E::MTK_HALFTONE_DISABLED as sys::MTK_HALFTONE_MODE_INDEX_T,
            MtkHalftone::Default => {
                E::MTK_HALFTONE_DEFAULT_CHECKER_SIZE as sys::MTK_HALFTONE_MODE_INDEX_T
            }
            MtkHalftone::Custom(size) => size,
        }
    }

    pub const fn from_raw(v: sys::MTK_HALFTONE_MODE_INDEX_T) -> MtkHalftone {
        use sys::MTK_HALFTONE_MODE_INDEX_E as E;
        match v {
            x if x == E::MTK_HALFTONE_DISABLED as sys::MTK_HALFTONE_MODE_INDEX_T => {
                MtkHalftone::Disabled
            }
            x if x == E::MTK_HALFTONE_DEFAULT_CHECKER_SIZE as sys::MTK_HALFTONE_MODE_INDEX_T => {
                MtkHalftone::Default
            }
            _ => MtkHalftone::Custom(v),
        }
    }
}

/// Framebuffer pixel format (`FBINK_PXFMT_INDEX_E`).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PixelFormat {
    Unknown,
    Y4,
    Y8,
    Bgr565,
    Rgb565,
    Bgr24,
    Rgb24,
    Bgra,
    Rgba,
    Bgr32,
    Rgb32,
}

impl PixelFormat {
    pub const fn to_raw(self) -> sys::FBINK_PXFMT_INDEX_T {
        use sys::FBINK_PXFMT_INDEX_E as E;
        match self {
            PixelFormat::Unknown => E::FBINK_PXFMT_UNKNOWN,
            PixelFormat::Y4 => E::FBINK_PXFMT_Y4,
            PixelFormat::Y8 => E::FBINK_PXFMT_Y8,
            PixelFormat::Bgr565 => E::FBINK_PXFMT_BGR565,
            PixelFormat::Rgb565 => E::FBINK_PXFMT_RGB565,
            PixelFormat::Bgr24 => E::FBINK_PXFMT_BGR24,
            PixelFormat::Rgb24 => E::FBINK_PXFMT_RGB24,
            PixelFormat::Bgra => E::FBINK_PXFMT_BGRA,
            PixelFormat::Rgba => E::FBINK_PXFMT_RGBA,
            PixelFormat::Bgr32 => E::FBINK_PXFMT_BGR32,
            PixelFormat::Rgb32 => E::FBINK_PXFMT_RGB32,
        }
    }

    pub const fn from_raw(v: sys::FBINK_PXFMT_INDEX_T) -> Option<PixelFormat> {
        use sys::FBINK_PXFMT_INDEX_E as E;
        match v {
            E::FBINK_PXFMT_UNKNOWN => Some(PixelFormat::Unknown),
            E::FBINK_PXFMT_Y4 => Some(PixelFormat::Y4),
            E::FBINK_PXFMT_Y8 => Some(PixelFormat::Y8),
            E::FBINK_PXFMT_BGR565 => Some(PixelFormat::Bgr565),
            E::FBINK_PXFMT_RGB565 => Some(PixelFormat::Rgb565),
            E::FBINK_PXFMT_BGR24 => Some(PixelFormat::Bgr24),
            E::FBINK_PXFMT_RGB24 => Some(PixelFormat::Rgb24),
            E::FBINK_PXFMT_BGRA => Some(PixelFormat::Bgra),
            E::FBINK_PXFMT_RGBA => Some(PixelFormat::Rgba),
            E::FBINK_PXFMT_BGR32 => Some(PixelFormat::Bgr32),
            E::FBINK_PXFMT_RGB32 => Some(PixelFormat::Rgb32),
            _ => None,
        }
    }
}

/// FBInk build target (`FBINK_TARGET_E`), selected at build time via the
/// `FBINK_TARGET` environment variable.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    strum::EnumCount,
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Target {
    /// Generic Linux (no eInk refresh support).
    Linux,
    Kobo,
    Kindle,
    /// Kindle using the original einkfb EPDC API.
    KindleLegacy,
    Cervantes,
    Remarkable,
    PocketBook,
}

impl Target {
    pub const fn to_raw(self) -> sys::FBINK_TARGET_T {
        use sys::FBINK_TARGET_E as E;
        match self {
            Target::Linux => E::FBINK_TARGET_LINUX,
            Target::Kobo => E::FBINK_TARGET_KOBO,
            Target::Kindle => E::FBINK_TARGET_KINDLE,
            Target::KindleLegacy => E::FBINK_TARGET_KINDLE_LEGACY,
            Target::Cervantes => E::FBINK_TARGET_CERVANTES,
            Target::Remarkable => E::FBINK_TARGET_REMARKABLE,
            Target::PocketBook => E::FBINK_TARGET_POCKETBOOK,
        }
    }

    pub const fn from_raw(v: sys::FBINK_TARGET_T) -> Option<Target> {
        use sys::FBINK_TARGET_E as E;
        match v {
            E::FBINK_TARGET_LINUX => Some(Target::Linux),
            E::FBINK_TARGET_KOBO => Some(Target::Kobo),
            E::FBINK_TARGET_KINDLE => Some(Target::Kindle),
            E::FBINK_TARGET_KINDLE_LEGACY => Some(Target::KindleLegacy),
            E::FBINK_TARGET_CERVANTES => Some(Target::Cervantes),
            E::FBINK_TARGET_REMARKABLE => Some(Target::Remarkable),
            E::FBINK_TARGET_POCKETBOOK => Some(Target::PocketBook),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_roundtrips() {
        for raw in 0..=sys::FONT_INDEX_E::COZETTE {
            let f = Font::from_raw(raw).expect("known font");
            assert_eq!(f.to_raw(), raw);
        }
        assert_eq!(Font::from_raw(sys::FONT_INDEX_E::FONT_MAX), None);
    }

    #[test]
    fn colors_match_across_fg_and_bg() {
        // Same gray level -> same actual grayscale value on both palettes.
        assert_eq!(FgColor::Gray1.to_raw(), 1); // 0x11
        assert_eq!(BgColor::Gray1.to_raw(), 14); // reversed palette
        assert_eq!(FgColor::Black.to_raw(), 0);
        assert_eq!(BgColor::Black.to_raw(), 15);
        for v in 0..=15u8 {
            assert!(FgColor::from_raw(v).is_some());
            assert!(BgColor::from_raw(v).is_some());
        }
    }

    #[test]
    fn sunxi_force_rota_is_signed() {
        assert_eq!(SunxiForceRota::NotSup.to_raw(), -128);
        assert_eq!(SunxiForceRota::Gyro.to_raw(), -1);
        assert_eq!(SunxiForceRota::Ur.to_raw(), 0);
        assert_eq!(
            SunxiForceRota::from_raw(-5),
            Some(SunxiForceRota::CurrentRota)
        );
        assert_eq!(SunxiForceRota::from_raw(42), None);
    }

    #[test]
    fn mtk_halftone_wraps_custom_sizes() {
        assert_eq!(MtkHalftone::Disabled.to_raw(), 0);
        assert_eq!(MtkHalftone::Default.to_raw(), 1);
        assert_eq!(MtkHalftone::Custom(9).to_raw(), 9);
        assert_eq!(MtkHalftone::from_raw(9), MtkHalftone::Custom(9));
        assert_eq!(MtkHalftone::from_raw(0), MtkHalftone::Disabled);
    }

    #[test]
    fn target_roundtrips() {
        for raw in 0..=sys::FBINK_TARGET_E::FBINK_TARGET_POCKETBOOK {
            let t = Target::from_raw(raw).expect("known target");
            assert_eq!(t.to_raw(), raw);
        }
        assert_eq!(Target::from_raw(200), None);
    }

    #[test]
    fn waveform_and_style_roundtrip() {
        assert_eq!(
            WaveformMode::from_raw(WaveformMode::Gl16.to_raw()),
            Some(WaveformMode::Gl16)
        );
        assert_eq!(FontStyle::BoldItalic.to_raw(), 3);
        assert_eq!(FontStyle::from_raw(2), Some(FontStyle::Bold));
    }

    #[test]
    fn strum_strings_roundtrip() {
        use strum::{EnumCount, IntoEnumIterator};
        assert_eq!(WaveformMode::Gc16.to_string(), "Gc16");
        assert_eq!("Gc16".parse(), Ok(WaveformMode::Gc16));
        assert_eq!(Font::UnsciiAlt.to_string(), "UnsciiAlt");
        assert_eq!("UnsciiAlt".parse(), Ok(Font::UnsciiAlt));
        assert!("Nope".parse::<Font>().is_err());
        // Iteration covers every variant, exactly once.
        assert_eq!(WaveformMode::iter().count(), WaveformMode::COUNT);
        assert_eq!(Font::iter().count(), Font::COUNT);
        assert!(WaveformMode::iter().any(|w| w == WaveformMode::Glrck16));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_uses_variant_names() {
        assert_eq!(
            serde_json::to_string(&WaveformMode::Gc16Fast).unwrap(),
            "\"Gc16Fast\""
        );
        assert_eq!(
            serde_json::from_str::<WaveformMode>("\"Gc16Fast\"").unwrap(),
            WaveformMode::Gc16Fast
        );
        // Data-carrying variants use serde's externally-tagged form.
        assert_eq!(
            serde_json::to_string(&MtkHalftone::Custom(3)).unwrap(),
            "{\"Custom\":3}"
        );
        assert_eq!(
            serde_json::from_str::<MtkHalftone>("{\"Custom\":3}").unwrap(),
            MtkHalftone::Custom(3)
        );
    }
}
