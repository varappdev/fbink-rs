//! [`State`], a safe snapshot of FBInk's internal state
//! (as returned by `fbink_get_state`).

use crate::enums::{NtxRotaQuirk, PixelFormat, SunxiForceRota};
use fbink_sys as sys;
use std::ffi::CStr;
use std::os::raw::c_char;

/// A snapshot of FBInk's state: device identification & quirks, screen & font
/// geometry, pen colors, etc.
///
/// Obtained via [`crate::Fbink::state`]. The C `char[]` arrays are converted
/// to `String`s, and the nullable `font_name` pointer to an `Option<String>`.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// `USER_HZ` (should pretty much always be 100).
    pub user_hz: i64,
    /// Name of the currently selected font.
    pub font_name: Option<String>,
    /// Viewport width (may differ from `screen_width` on devices with a viewport).
    pub view_width: u32,
    /// Viewport height (ditto).
    pub view_height: u32,
    /// Effective screen width.
    pub screen_width: u32,
    /// Effective screen height.
    pub screen_height: u32,
    /// Scanline length in bytes (padding included).
    pub scanline_stride: u32,
    /// Bits per pixel.
    pub bpp: u32,
    /// True if the fb uses `GRAYSCALE_8BIT_INVERTED` (at 8bpp).
    pub inverted_grayscale: bool,
    /// Short common device name (no brand).
    pub device_name: String,
    /// Device codename.
    pub device_codename: String,
    /// Device platform (often a codename, too).
    pub device_platform: String,
    /// Decimal device ID (c.f. `fbink_device_id.c`; Kindles have none).
    pub device_id: u16,
    /// Actual grayscale value of the foreground pen (not a palette index).
    pub pen_fg_color: u8,
    /// Actual grayscale value of the background pen (not a palette index).
    pub pen_bg_color: u8,
    /// Screen DPI.
    pub screen_dpi: u16,
    /// Effective width of a glyph cell (i.e. scaled).
    pub font_w: u16,
    /// Effective height of a glyph cell (i.e. scaled).
    pub font_h: u16,
    /// Max columns at the current cell size.
    pub max_cols: u16,
    /// Max rows at the current cell size.
    pub max_rows: u16,
    /// Non-zero on devices with a horizontal viewport.
    pub view_hori_origin: u8,
    /// Origin in px of row 0 (includes viewport + `view_vert_offset`).
    pub view_vert_origin: u8,
    /// Shift in px needed to vertically balance rows over `view_height`.
    pub view_vert_offset: u8,
    /// Current cell scaling multiplier.
    pub fontsize_mult: u8,
    /// Native width of a glyph cell (i.e. unscaled).
    pub glyph_width: u8,
    /// Native height of a glyph cell (i.e. unscaled).
    pub glyph_height: u8,
    /// Horizontal column balance is perfect over `view_width`.
    pub is_perfect_fit: bool,
    /// Device runs on a MediaTek SoC.
    pub is_mtk: bool,
    /// Device runs on an AllWinner (sunxi) SoC.
    pub is_sunxi: bool,
    /// Device is a Tolino running a firmware in Tolino mode.
    pub is_tolino: bool,
    /// The sunxi fbdamage module is loaded.
    pub sunxi_has_fbdamage: bool,
    /// Current effective sunxi rotation policy.
    pub sunxi_force_rota: SunxiForceRota,
    /// Device is a Kindle using the original einkfb EPDC API.
    pub is_kindle_legacy: bool,
    /// Device is a Kobo with no MultiTouch input support.
    pub is_kobo_non_mt: bool,
    /// `MXCFB_WAIT_FOR_UPDATE_COMPLETE` may timeout on this device.
    pub unreliable_wait_for: bool,
    /// `fbink_wakeup_epdc` is *not* a NOP on this device.
    pub can_wake_epdc: bool,
    /// Native rotation at boot.
    pub ntx_boot_rota: u8,
    /// NTX kernel rotation quirk type.
    pub ntx_rota_quirk: NtxRotaQuirk,
    /// Native -> canonical rotation map (index is native, value canonical).
    pub rotation_map: [u8; 4],
    /// Touch panel reports swapped coordinate axes (handle this first).
    pub touch_swap_axes: bool,
    /// Post-swap, touch panel reports inverted x coordinates.
    pub touch_mirror_x: bool,
    /// Post-swap, touch panel reports inverted y coordinates.
    pub touch_mirror_y: bool,
    /// Rotation compensation is in effect (the isNTX16bLandscape quirk).
    pub is_ntx_quirky_landscape: bool,
    /// Current native rotation (device rotation, not the buffer's!).
    pub current_rota: u8,
    /// Device has a gyro.
    pub can_rotate: bool,
    /// Device can use EPDC inversion.
    pub can_hw_invert: bool,
    /// Device can use nightmode ("eclipse") waveform modes.
    pub has_eclipse_wfm: bool,
    /// Device features a Kaleido/CFA color panel.
    pub has_color_panel: bool,
    /// Framebuffer pixel format.
    pub pixel_format: PixelFormat,
    /// Device supports `fbink_wait_for_submission`.
    pub can_wait_for_submission: bool,
}

impl State {
    /// Convert from the raw `fbink-sys` state struct.
    pub(crate) fn from_raw(raw: &sys::FBInkState) -> Self {
        Self {
            // c_long is 32-bit on our 32-bit ARM targets, so this cast *is*
            // necessary there (clippy only sees the 64-bit host layout).
            #[allow(clippy::unnecessary_cast)]
            user_hz: raw.user_hz as i64,
            font_name: c_str_opt(raw.font_name),
            view_width: raw.view_width,
            view_height: raw.view_height,
            screen_width: raw.screen_width,
            screen_height: raw.screen_height,
            scanline_stride: raw.scanline_stride,
            bpp: raw.bpp,
            inverted_grayscale: raw.inverted_grayscale,
            device_name: c_array_str(&raw.device_name),
            device_codename: c_array_str(&raw.device_codename),
            device_platform: c_array_str(&raw.device_platform),
            device_id: raw.device_id,
            pen_fg_color: raw.pen_fg_color,
            pen_bg_color: raw.pen_bg_color,
            screen_dpi: raw.screen_dpi,
            font_w: raw.font_w,
            font_h: raw.font_h,
            max_cols: raw.max_cols,
            max_rows: raw.max_rows,
            view_hori_origin: raw.view_hori_origin,
            view_vert_origin: raw.view_vert_origin,
            view_vert_offset: raw.view_vert_offset,
            fontsize_mult: raw.fontsize_mult,
            glyph_width: raw.glyph_width,
            glyph_height: raw.glyph_height,
            is_perfect_fit: raw.is_perfect_fit,
            is_mtk: raw.is_mtk,
            is_sunxi: raw.is_sunxi,
            is_tolino: raw.is_tolino,
            sunxi_has_fbdamage: raw.sunxi_has_fbdamage,
            sunxi_force_rota: SunxiForceRota::from_raw(raw.sunxi_force_rota)
                .unwrap_or(SunxiForceRota::NotSup),
            is_kindle_legacy: raw.is_kindle_legacy,
            is_kobo_non_mt: raw.is_kobo_non_mt,
            unreliable_wait_for: raw.unreliable_wait_for,
            can_wake_epdc: raw.can_wake_epdc,
            ntx_boot_rota: raw.ntx_boot_rota,
            ntx_rota_quirk: NtxRotaQuirk::from_raw(raw.ntx_rota_quirk)
                .unwrap_or(NtxRotaQuirk::Straight),
            rotation_map: raw.rotation_map,
            touch_swap_axes: raw.touch_swap_axes,
            touch_mirror_x: raw.touch_mirror_x,
            touch_mirror_y: raw.touch_mirror_y,
            is_ntx_quirky_landscape: raw.is_ntx_quirky_landscape,
            current_rota: raw.current_rota,
            can_rotate: raw.can_rotate,
            can_hw_invert: raw.can_hw_invert,
            has_eclipse_wfm: raw.has_eclipse_wfm,
            has_color_panel: raw.has_color_panel,
            pixel_format: PixelFormat::from_raw(raw.pixel_format).unwrap_or(PixelFormat::Unknown),
            can_wait_for_submission: raw.can_wait_for_submission,
        }
    }
}

/// Convert a NUL-terminated `char[]` field to a `String`.
fn c_array_str(arr: &[c_char]) -> String {
    // Safety: `char` and `u8` have the same layout; the array is a fixed-size
    // C string field, NUL-terminated (we stop at the first NUL anyway).
    let bytes = unsafe { std::slice::from_raw_parts(arr.as_ptr().cast::<u8>(), arr.len()) };
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// Convert a nullable `const char*` to an `Option<String>`.
fn c_str_opt(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        // Safety: FBInk points at a static, NUL-terminated font name.
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_array_stops_at_nul() {
        let mut arr = [0 as c_char; 32];
        for (i, b) in b"kobo".iter().enumerate() {
            arr[i] = *b as c_char;
        }
        assert_eq!(c_array_str(&arr), "kobo");
    }

    #[test]
    fn c_array_tolerates_missing_nul() {
        let arr = [b'x' as c_char; 4];
        assert_eq!(c_array_str(&arr), "xxxx");
    }

    #[test]
    fn null_pointer_maps_to_none() {
        assert_eq!(c_str_opt(std::ptr::null()), None);
    }
}
