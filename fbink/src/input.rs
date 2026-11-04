//! Input device scanning utilities (`input` feature).
//!
//! [`scan`] & [`check`] return heap-allocated data from FBInk which must be
//! `free()`d; the wrappers copy it into owned Rust values and free the C
//! allocation right away. Any device fd FBInk leaves open is wrapped in an
//! [`OwnedFd`], which closes it on drop.

use crate::error::{Error, Result, cstring};
use fbink_sys as sys;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::raw::{c_char, c_void};

unsafe extern "C" {
    // FBInk allocates the scan results with malloc(): free them with libc's
    // free(). Declared by hand to avoid pulling in `libc`.
    fn free(ptr: *mut c_void);
}

/// Bitmask of input device types, as classified by FBInk/udev
/// (`INPUT_DEVICE_TYPE_E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct InputDeviceType(u32);

impl InputDeviceType {
    pub const UNKNOWN: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_UNKNOWN);
    pub const POINTINGSTICK: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_POINTINGSTICK);
    pub const MOUSE: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_MOUSE);
    pub const TOUCHPAD: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_TOUCHPAD);
    pub const TOUCHSCREEN: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_TOUCHSCREEN);
    pub const JOYSTICK: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_JOYSTICK);
    /// Includes touchscreens with stylus support.
    pub const TABLET: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_TABLET);
    /// Supports at least one `EV_KEY` keycode (may include touchscreens).
    pub const KEY: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_KEY);
    pub const KEYBOARD: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_KEYBOARD);
    pub const ACCELEROMETER: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_ACCELEROMETER);
    pub const POWER_BUTTON: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_POWER_BUTTON);
    pub const SLEEP_COVER: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_SLEEP_COVER);
    pub const PAGINATION_BUTTONS: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_PAGINATION_BUTTONS);
    pub const HOME_BUTTON: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_HOME_BUTTON);
    /// e.g. the frontlight toggle button on some early Kobos.
    pub const LIGHT_BUTTON: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_LIGHT_BUTTON);
    pub const MENU_BUTTON: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_MENU_BUTTON);
    pub const DPAD: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_DPAD);
    /// Where "device was rotated" events may be sent (gyro or not).
    pub const ROTATION_EVENT: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_ROTATION_EVENT);
    /// [`InputDeviceType::TABLET`], with max `ABS_X`/`ABS_Y` matching the fb.
    pub const SCALED_TABLET: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_SCALED_TABLET);
    pub const VOLUME_BUTTONS: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_VOLUME_BUTTONS);
    /// For the "double-tap on device" gestures (Kindle PW6 & CS).
    pub const KINDLE_FRAME_TAP: Self = Self(sys::INPUT_DEVICE_TYPE_E::INPUT_KINDLE_FRAME_TAP);

    /// The raw `INPUT_DEVICE_TYPE_*` bitmask.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// True if *all* the bits in `other` are set here.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for InputDeviceType {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Bitmask of scan behavior settings (`INPUT_SETTINGS_TYPE_E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct InputSettings(u32);

impl InputSettings {
    /// Default settings: match any bit, leave fds open (non-blocking).
    pub const DEFAULT: Self = Self(0);
    /// Do *not* leave any fd open.
    pub const SCAN_ONLY: Self = Self(sys::INPUT_SETTINGS_TYPE_E::SCAN_ONLY as u32);
    /// Open fds in *blocking* mode (default is `O_RDONLY|O_NONBLOCK|O_CLOEXEC`).
    pub const OPEN_BLOCKING: Self = Self(sys::INPUT_SETTINGS_TYPE_E::OPEN_BLOCKING as u32);
    /// A device must feature *all* the `match_types` bits to match.
    pub const MATCH_ALL: Self = Self(sys::INPUT_SETTINGS_TYPE_E::MATCH_ALL as u32);
    /// A device must not feature *all* the `exclude_types` bits to match.
    pub const EXCLUDE_ALL: Self = Self(sys::INPUT_SETTINGS_TYPE_E::EXCLUDE_ALL as u32);
    /// Do *not* print FBInk's recap of the input device classification.
    pub const NO_RECAP: Self = Self(sys::INPUT_SETTINGS_TYPE_E::NO_RECAP as u32);

    /// The raw `INPUT_SETTINGS_TYPE_*` bitmask.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// True if *all* the bits in `other` are set here.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for InputSettings {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// A scanned input device (as classified by FBInk).
#[derive(Debug)]
pub struct InputDevice {
    /// The device's type classification (bitmask).
    pub type_: InputDeviceType,
    /// The device's open fd, unless [`InputSettings::SCAN_ONLY`] was used (or
    /// FBInk couldn't open it). Closed when this value is dropped.
    pub fd: Option<OwnedFd>,
    /// True if the type matched the `match_types`/`exclude_types` combo.
    pub matched: bool,
    /// As reported by `EVIOCGNAME`.
    pub name: String,
    /// e.g. `/dev/input/event0`.
    pub path: String,
}

impl InputDevice {
    fn from_raw(raw: &sys::FBInkInputDevice) -> Self {
        Self {
            type_: InputDeviceType(raw.type_),
            // Safety: FBInk opened the fd itself and hands over ownership;
            // negative values mean "not open".
            fd: (raw.fd >= 0).then(|| unsafe { OwnedFd::from_raw_fd(raw.fd) }),
            matched: raw.matched,
            name: c_array_str(&raw.name),
            path: c_array_str(&raw.path),
        }
    }
}

/// Scan & classify the system's input devices (`/dev/input/event*`).
///
/// The result contains *all* the devices, with [`InputDevice::matched`] set
/// according to `match_types`/`exclude_types` (see [`InputSettings`] for how
/// the masks combine). Device fds are left open unless
/// [`InputSettings::SCAN_ONLY`] is set (they are closed on drop).
///
/// NOTE: does not require FBInk to be initialized, but honors its verbosity.
pub fn scan(
    match_types: InputDeviceType,
    exclude_types: InputDeviceType,
    settings: InputSettings,
) -> Result<Vec<InputDevice>> {
    let mut dev_count: usize = 0;
    // Safety: dev_count is a valid out-pointer.
    let ptr = unsafe {
        sys::fbink_input_scan(
            match_types.raw(),
            exclude_types.raw(),
            settings.raw(),
            &mut dev_count,
        )
    };
    if ptr.is_null() {
        // No input devices could be read.
        return Err(Error::NoData);
    }
    // Safety: FBInk returned a malloc'd array of `dev_count` elements.
    let raws = unsafe { std::slice::from_raw_parts(ptr, dev_count) };
    let devices = raws.iter().map(InputDevice::from_raw).collect();
    // Safety: the array came from malloc() and is not used afterwards.
    unsafe { free(ptr.cast::<c_void>()) };
    Ok(devices)
}

/// Classify a single input device from its file path (e.g. to handle hotplug
/// events), with the same matching semantics as [`scan`].
pub fn check(
    filepath: &str,
    match_types: InputDeviceType,
    exclude_types: InputDeviceType,
    settings: InputSettings,
) -> Result<InputDevice> {
    let filepath = cstring(filepath)?;
    // Safety: filepath is a valid C string.
    let ptr = unsafe {
        sys::fbink_input_check(
            filepath.as_ptr(),
            match_types.raw(),
            exclude_types.raw(),
            settings.raw(),
        )
    };
    if ptr.is_null() {
        // The file could not be read.
        return Err(Error::NoData);
    }
    // Safety: FBInk returned a malloc'd single struct.
    let device = InputDevice::from_raw(unsafe { &*ptr });
    // Safety: the struct came from malloc() and is not used afterwards.
    unsafe { free(ptr.cast::<c_void>()) };
    Ok(device)
}

/// Convert a NUL-terminated `char[]` field to a `String`.
fn c_array_str(arr: &[c_char]) -> String {
    // Safety: `char` and `u8` have the same layout; we stop at the first NUL.
    let bytes = unsafe { std::slice::from_raw_parts(arr.as_ptr().cast::<u8>(), arr.len()) };
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    // Valid UTF-8 is expected, but we're not going to trust the kernel for it.
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_masks_combine() {
        let mask = InputDeviceType::TOUCHSCREEN | InputDeviceType::KEY;
        assert!(mask.contains(InputDeviceType::TOUCHSCREEN));
        assert!(mask.contains(InputDeviceType::KEY));
        assert!(!mask.contains(InputDeviceType::TOUCHSCREEN | InputDeviceType::MOUSE));
    }

    #[test]
    fn settings_combine() {
        let s = InputSettings::SCAN_ONLY | InputSettings::NO_RECAP;
        assert!(s.contains(InputSettings::SCAN_ONLY));
        assert!(!s.contains(InputSettings::OPEN_BLOCKING));
        assert!(!InputSettings::DEFAULT.contains(InputSettings::SCAN_ONLY));
    }

    #[test]
    fn c_array_stops_at_nul() {
        let mut arr = [0 as c_char; 8];
        for (i, b) in b"event0".iter().enumerate() {
            arr[i] = *b as c_char;
        }
        assert_eq!(c_array_str(&arr), "event0");
    }
}
