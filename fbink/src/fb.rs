//! The [`Fbink`] handle: RAII access to the framebuffer & all fd-taking calls.

use crate::config::Config;
use crate::error::{check, check_nonneg, cstring, Error, Result, ENOSYS, ERANGE};
use crate::rect::Rect;
use crate::state::State;
use fbink_sys as sys;
use std::os::raw::c_int;

/// Pass this as a marker to the `wait_for_*` calls to wait for the last
/// update sent by this FBInk session.
pub const LAST_MARKER: u32 = sys::LAST_MARKER;

/// Magic `rota` value for [`Fbink::set_fb_info`]: leave the rotation untouched.
pub const KEEP_CURRENT_ROTATE: u32 = sys::KEEP_CURRENT_ROTATE;
/// Magic `bpp` value for [`Fbink::set_fb_info`]: leave the bitdepth untouched.
pub const KEEP_CURRENT_BITDEPTH: u8 = sys::KEEP_CURRENT_BITDEPTH as u8;
/// Magic `grayscale` value for [`Fbink::set_fb_info`]: leave it untouched.
pub const KEEP_CURRENT_GRAYSCALE: u8 = sys::KEEP_CURRENT_GRAYSCALE as u8;
/// Magic `grayscale` value for [`Fbink::set_fb_info`]: toggle 8bpp inversion.
pub const TOGGLE_GRAYSCALE: u8 = sys::TOGGLE_GRAYSCALE as u8;

/// What changed during a successful [`Fbink::reinit`] (or
/// [`Fbink::set_fb_info`]/[`Fbink::sunxi_ntx_enforce_rota`], which return the
/// same bitmask): `0` means nothing changed, any flag set means FBInk's
/// internal state (and thus anything you cached from
/// [`crate::Fbink::state`] or [`Fbink::fb_pointer`]) may be stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReinitChanges(u32);

impl ReinitChanges {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The raw `OK_*_CHANGE` bitmask.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// No reinitialization was needed.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The bitdepth changed.
    pub const fn bpp_changed(self) -> bool {
        self.0 & sys::OK_BPP_CHANGE != 0
    }

    /// The rotation changed.
    pub const fn rota_changed(self) -> bool {
        self.0 & sys::OK_ROTA_CHANGE != 0
    }

    /// A rotation change swapped the layout (portrait <-> landscape).
    ///
    /// Implies [`ReinitChanges::rota_changed`]; if *only* `rota_changed` is
    /// set, it was a simple inversion of the current orientation.
    pub const fn layout_changed(self) -> bool {
        self.0 & sys::OK_LAYOUT_CHANGE != 0
    }

    /// The grayscale flag changed (8bpp only).
    ///
    /// On mxcfb-like platforms this toggles global HW inversion (night mode):
    /// you may want to react to it (e.g. change pen colors).
    pub const fn grayscale_changed(self) -> bool {
        self.0 & sys::OK_GRAYSCALE_CHANGE != 0
    }
}

/// The set of features compiled into the FBInk library, as returned by
/// [`crate::features`] (mirrors the `fbink-sys` cargo features).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features(u32);

impl Features {
    /// Bare MINIMAL build: no text, no drawing, no images.
    pub const MINIMAL: Self = Self(sys::FBINK_FEATURE_MINIMAL);
    /// Basic draw primitives.
    pub const DRAW: Self = Self(sys::FBINK_FEATURE_DRAW);
    /// Fixed-cell font rendering + the base IBM font.
    pub const BITMAP: Self = Self(sys::FBINK_FEATURE_BITMAP);
    /// The full set of bundled fixed-cell fonts.
    pub const FONTS: Self = Self(sys::FBINK_FEATURE_FONTS);
    /// GNU Unifont for the fixed-cell font rendering.
    pub const UNIFONT: Self = Self(sys::FBINK_FEATURE_UNIFONT);
    /// TrueType/OpenType font rendering.
    pub const OPENTYPE: Self = Self(sys::FBINK_FEATURE_OPENTYPE);
    /// Image (& framebuffer dump) support.
    pub const IMAGE: Self = Self(sys::FBINK_FEATURE_IMAGE);
    /// Kobo-only USBMS "Connect" button scanner (deprecated).
    pub const BUTTON_SCAN: Self = Self(sys::FBINK_FEATURE_BUTTON_SCAN);
    /// Input device scanning utilities.
    pub const INPUT: Self = Self(sys::FBINK_FEATURE_INPUT);
    /// The standard full-featured build.
    pub const FULL: Self = Self(sys::FBINK_FEATURE_FULL);

    /// The raw `FBINK_FEATURE_*` bitmask.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Wrap a raw `FBINK_FEATURE_*` bitmask.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// True if no feature at all is enabled (MINIMAL build).
    pub const fn is_empty(self) -> bool {
        self.0 == Self::MINIMAL.0
    }

    /// True if *all* the bits in `other` are enabled.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for Features {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// A handle to the framebuffer.
///
/// Two ways to get one:
///
/// * [`Fbink::open`]: opens & owns the framebuffer file descriptor (closed on
///   [`Drop`]). Keeps FBInk's mmap around for the lifetime of the handle: the
///   fastest path for a long-running application.
/// * [`Fbink::auto`]: uses FBInk's `FBFD_AUTO`, letting FBInk open (and close)
///   the fb per call. Nothing to release.
///
/// Most calls require [`Fbink::init`] to have been called at least once first.
#[derive(Debug)]
pub struct Fbink {
    fbfd: c_int,
    owned: bool,
}

impl Fbink {
    /// Open the framebuffer character device; the descriptor is closed when
    /// the handle is dropped.
    pub fn open() -> Result<Self> {
        // Safety: plain call, no preconditions.
        let fbfd = unsafe { sys::fbink_open() };
        if fbfd < 0 {
            // fbink_open() is a thin open() wrapper: the reason is in errno.
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(1);
            return Err(Error::from_errno(errno));
        }
        Ok(Self { fbfd, owned: true })
    }

    /// A handle using `FBFD_AUTO`: FBInk opens the fb for the duration of each
    /// call. Nothing is owned or closed by this handle.
    pub fn auto() -> Self {
        Self {
            fbfd: sys::FBFD_AUTO,
            owned: false,
        }
    }

    /// The underlying framebuffer file descriptor (`FBFD_AUTO` == `-1` for
    /// [`Fbink::auto`]).
    pub fn fd(&self) -> c_int {
        self.fbfd
    }

    /// Close the framebuffer now instead of at drop time, surfacing errors.
    ///
    /// NOTE: on failure the fd may or may not still be open (FBInk's error
    /// paths differ on whether the close already happened); it is never
    /// retried, so a failed close can leak the fd.
    pub fn close(mut self) -> Result<()> {
        // Safety: FBInk tolerates closing any fbfd, including FBFD_AUTO.
        let ret = unsafe { sys::fbink_close(self.fbfd) };
        self.owned = false;
        check(ret)
    }

    /// Initialize FBInk's internal state. **Must** be called at least once
    /// before any other call (bar the handful of global helpers).
    ///
    /// Honors `is_centered`, `fontmult`, `fontname`, `fg_color`, `bg_color`,
    /// `no_viewport`, `is_verbose`, `is_quiet` & `to_syslog` from `cfg`; those
    /// "stick" until the next (re)init. Prefer [`Fbink::reinit`] to refresh
    /// state after a framebuffer change.
    ///
    /// NOTE: `fbink_init` is *not* thread-safe (it sets global state): call it
    /// from your main thread before spawning threads. Everything else is.
    pub fn init(&self, cfg: &Config) -> Result<()> {
        // Safety: cfg is a valid reference; fbfd is valid or FBFD_AUTO.
        check(unsafe { sys::fbink_init(self.fbfd, cfg.raw()) })
    }

    /// Detect framebuffer state changes since the last (re)init, doing the
    /// least amount of work possible.
    ///
    /// Returns what changed: `0` (empty) means nothing to do. Cannot replace
    /// the initial [`Fbink::init`] call.
    pub fn reinit(&self, cfg: &Config) -> Result<ReinitChanges> {
        // Safety: see Fbink::init.
        let ret = unsafe { sys::fbink_reinit(self.fbfd, cfg.raw()) };
        if ret < 0 {
            Err(Error::from_ret(ret))
        } else {
            Ok(ReinitChanges::from_raw(ret as u32))
        }
    }

    /// Snapshot FBInk's state (device quirks, screen & font geometry, ...).
    pub fn state(&self, cfg: &Config) -> State {
        let mut raw: sys::FBInkState = unsafe { std::mem::zeroed() };
        // Safety: both pointers are valid; the struct is scalar-only.
        unsafe { sys::fbink_get_state(cfg.raw(), &mut raw) };
        State::from_raw(&raw)
    }

    /// Dump FBInk's state to stdout in a shell-consumable (`eval`) format.
    ///
    /// # Upstream bug (patched at build time)
    ///
    /// At the pinned FBInk revision (`886f25f`), `fbink_state_dump`'s
    /// `fprintf` passes only two arguments for three format specifiers on
    /// non-Kobo builds (the `#else` branch of the `isTolino`/`SunxiHasFBDamage`/
    /// `SunxiForceRota` triplet). The remaining values shift by one, so
    /// `pixelFormat='%s'` receives an integer (0 or 1) as a pointer and the
    /// call **segfaults the process**. Reproduced on a Kindle Basic 5 (22D)
    /// with the stock FBInk 1.25.0 binary (`fbink -e`); still unfixed on
    /// upstream `master`.
    ///
    /// `fbink-sys` applies a fix to the bundled source at build time
    /// (see `fbink-sys/patches/`), so calls through this crate are safe; the
    /// crash only reproduces with unpatched FBInk builds.
    pub fn state_dump(&self, cfg: &Config) {
        // Safety: cfg is a valid reference.
        unsafe { sys::fbink_state_dump(cfg.raw()) };
    }

    /// Print a UTF-8 string on screen.
    ///
    /// Returns the amount of lines printed. Fails with [`Error::Invalid`] on
    /// an empty string, [`Error::InvalidUtf8`] on invalid UTF-8 (a Rust `&str`
    /// cannot actually trigger that) and [`Error::NotSupported`] on a MINIMAL
    /// build without fixed-cell font support.
    pub fn print(&self, text: &str, cfg: &Config) -> Result<u32> {
        let text = cstring(text)?;
        // Safety: text is a valid C string for the duration of the call.
        check_nonneg(unsafe { sys::fbink_print(self.fbfd, text.as_ptr(), cfg.raw()) })
    }

    /// Print a full-width progress bar (`percentage` in 0-100).
    pub fn print_progress_bar(&self, percentage: u8, cfg: &Config) -> Result<()> {
        // Safety: cfg is a valid reference.
        check(unsafe { sys::fbink_print_progress_bar(self.fbfd, percentage, cfg.raw()) })
    }

    /// Print a full-width activity bar (an infinite progress bar), with the
    /// thumb at `progress` (0-16).
    pub fn print_activity_bar(&self, progress: u8, cfg: &Config) -> Result<()> {
        // Safety: cfg is a valid reference.
        check(unsafe { sys::fbink_print_activity_bar(self.fbfd, progress, cfg.raw()) })
    }

    /// Print an image file on screen (JPEG, PNG, TGA, BMP, GIF & PNM).
    ///
    /// `path` may be `"-"` to read image data from stdin (when not a tty).
    /// Honors `halign`/`valign`, `row`/`col` & `x_off`/`y_off` for positioning.
    pub fn print_image(&self, path: &str, x_off: i16, y_off: i16, cfg: &Config) -> Result<()> {
        let path = cstring(path)?;
        // Safety: path is a valid C string for the duration of the call.
        check(unsafe { sys::fbink_print_image(self.fbfd, path.as_ptr(), x_off, y_off, cfg.raw()) })
    }

    /// Print raw packed-pixel scanlines on screen.
    ///
    /// `data` holds `Y`/`YA`/`RGB`/`RGBA` 8-bit components (first pixel at the
    /// top-left); the input pixel format is computed as
    /// `data.len() / h / w`, so `data` must be *exactly* the image, with no
    /// padding.
    pub fn print_raw_data(
        &self,
        data: &[u8],
        w: i32,
        h: i32,
        x_off: i16,
        y_off: i16,
        cfg: &Config,
    ) -> Result<()> {
        if w <= 0 || h <= 0 {
            return Err(Error::Invalid);
        }
        // The input pixel format is derived as len / h / w, so the buffer must
        // be an exact, unpadded 1-to-4-component image (Y/YA/RGB/RGBA).
        let pixels = w as u64 * h as u64;
        let len = data.len() as u64;
        if len == 0 || len % pixels != 0 || len / pixels > 4 {
            return Err(Error::Invalid);
        }
        // Safety: data points at `data.len()` readable bytes.
        check(unsafe {
            sys::fbink_print_raw_data(
                self.fbfd,
                data.as_ptr(),
                w,
                h,
                data.len(),
                x_off,
                y_off,
                cfg.raw(),
            )
        })
    }

    /// Trigger an eInk refresh of a region (honors `wfm_mode`,
    /// `dithering_mode`, `is_nightmode` & `is_flashing` from `cfg`).
    ///
    /// NOTE: coordinates follow the *mxcfb* order (top before left!). An empty
    /// region (`0x0 @ (0, 0)`) means a full-screen refresh. This ignores
    /// `no_refresh`, and returns [`Error::NotSupported`] on pure Linux builds.
    pub fn refresh(
        &self,
        top: u32,
        left: u32,
        width: u32,
        height: u32,
        cfg: &Config,
    ) -> Result<()> {
        // Safety: cfg is a valid reference.
        check(unsafe { sys::fbink_refresh(self.fbfd, top, left, width, height, cfg.raw()) })
    }

    /// Same as [`Fbink::refresh`], with an [`Rect`] (whose `left` comes first!)
    /// instead of broken-out coordinates.
    pub fn refresh_rect(&self, rect: &Rect, cfg: &Config) -> Result<()> {
        let raw = rect.to_raw();
        // Safety: raw is a valid reference.
        check(unsafe { sys::fbink_refresh_rect(self.fbfd, &raw, cfg.raw()) })
    }

    /// Wait for the submission of an update marker
    /// ([`LAST_MARKER`] for the last one sent by this session).
    ///
    /// Only implemented by Kindle kernels (K5+) & Kobo MTK (Mk. 11).
    pub fn wait_for_submission(&self, marker: u32) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_wait_for_submission(self.fbfd, marker) })
    }

    /// Wait for the completion of an update marker
    /// ([`LAST_MARKER`] for the last one sent by this session).
    ///
    /// Behaves like a vsync fence; the wait is longer than the visual refresh.
    pub fn wait_for_complete(&self, marker: u32) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_wait_for_complete(self.fbfd, marker) })
    }

    /// The marker of the last refresh sent by this FBInk session
    /// ([`LAST_MARKER`] when there was none).
    pub fn last_marker(&self) -> u32 {
        crate::get_last_marker()
    }

    /// Clear the screen (or `rect`'s region) using the background pen color,
    /// with an eInk refresh (unless `no_refresh`).
    ///
    /// `rect` in absolute coordinates: an empty rect or `None` clears the full
    /// screen. `no_rota` only matters in the quirky rotated-rect case: when in
    /// doubt, `false`.
    pub fn cls(&self, cfg: &Config, rect: Option<&Rect>, no_rota: bool) -> Result<()> {
        let raw = rect.map(|r| r.to_raw());
        let raw_ptr = raw
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const sys::FBInkRect);
        // Safety: cfg is a valid reference; raw_ptr is null or valid.
        check(unsafe { sys::fbink_cls(self.fbfd, cfg.raw(), raw_ptr, no_rota) })
    }

    /// Like [`Fbink::cls`], but in grid (columns/rows) coordinates, honoring
    /// the same positioning trickery as [`Fbink::print`].
    pub fn grid_clear(&self, cols: u16, rows: u16, cfg: &Config) -> Result<()> {
        // Safety: cfg is a valid reference.
        check(unsafe { sys::fbink_grid_clear(self.fbfd, cols, rows, cfg.raw()) })
    }

    /// Like [`Fbink::refresh`], but in grid (columns/rows) coordinates.
    /// Ignores `no_refresh`.
    pub fn grid_refresh(&self, cols: u16, rows: u16, cfg: &Config) -> Result<()> {
        // Safety: cfg is a valid reference.
        check(unsafe { sys::fbink_grid_refresh(self.fbfd, cols, rows, cfg.raw()) })
    }

    /// Invert the *existing* content of the full screen (and refresh).
    ///
    /// Useful on devices without HW inversion, to swap to/from nightmode
    /// without redrawing anything.
    pub fn invert_screen(&self, cfg: &Config) -> Result<()> {
        // Safety: cfg is a valid reference.
        check(unsafe { sys::fbink_invert_screen(self.fbfd, cfg.raw()) })
    }

    /// Invert the *existing* content of `rect`'s region (no refresh!).
    ///
    /// An empty rect or `None` inverts the full screen. `no_rota`: see
    /// [`Fbink::cls`].
    pub fn invert_rect(&self, rect: Option<&Rect>, no_rota: bool) -> Result<()> {
        let raw = rect.map(|r| r.to_raw());
        let raw_ptr = raw
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const sys::FBInkRect);
        // Safety: raw_ptr is null or valid.
        check(unsafe { sys::fbink_invert_rect(self.fbfd, raw_ptr, no_rota) })
    }

    /// Fill a rectangle with an 8-bit luminance value (honors `no_refresh` &
    /// `is_inverted`). An empty rect or `None` fills the full screen.
    pub fn fill_rect_gray(
        &self,
        cfg: &Config,
        rect: Option<&Rect>,
        no_rota: bool,
        y: u8,
    ) -> Result<()> {
        let raw = rect.map(|r| r.to_raw());
        let raw_ptr = raw
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const sys::FBInkRect);
        // Safety: cfg is a valid reference; raw_ptr is null or valid.
        check(unsafe { sys::fbink_fill_rect_gray(self.fbfd, cfg.raw(), raw_ptr, no_rota, y) })
    }

    /// Fill a rectangle with an RGBA color (`a` opaque is `0xFF`).
    /// Honors `no_refresh` & `is_inverted`. An empty rect or `None` fills the
    /// full screen.
    #[allow(clippy::too_many_arguments)] // mirrors the C signature
    pub fn fill_rect_rgba(
        &self,
        cfg: &Config,
        rect: Option<&Rect>,
        no_rota: bool,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) -> Result<()> {
        let raw = rect.map(|r| r.to_raw());
        let raw_ptr = raw
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const sys::FBInkRect);
        // Safety: cfg is a valid reference; raw_ptr is null or valid.
        check(unsafe {
            sys::fbink_fill_rect_rgba(self.fbfd, cfg.raw(), raw_ptr, no_rota, r, g, b, a)
        })
    }

    /// Set a single pixel to an 8-bit luminance value.
    ///
    /// Convenience over performance (the pixel is packed on every call): the
    /// [`crate::pack_pixel_gray`] + [`Fbink::put_pixel`] path is faster for
    /// repeated colors. Never triggers a refresh.
    pub fn put_pixel_gray(&self, x: u16, y: u16, v: u8) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_put_pixel_gray(self.fbfd, x, y, v) })
    }

    /// Set a single pixel to an RGBA color (`a` opaque is `0xFF`).
    /// See [`Fbink::put_pixel_gray`] for the performance caveats.
    pub fn put_pixel_rgba(&self, x: u16, y: u16, r: u8, g: u8, b: u8, a: u8) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_put_pixel_rgba(self.fbfd, x, y, r, g, b, a) })
    }

    /// Set a single pixel to a pre-packed pixel value
    /// (from [`crate::pack_pixel_gray`]/[`crate::pack_pixel_rgba`]).
    ///
    /// NOTE: packing is only accurate for the *current* pixel format: re-pack
    /// after a bitdepth change (e.g. on a non-empty [`Fbink::reinit`]).
    pub fn put_pixel(&self, x: u16, y: u16, px: PackedPixel) -> Result<()> {
        let mut px = px.0;
        // Safety: px points at a valid u32 for the duration of the call.
        check(unsafe {
            sys::fbink_put_pixel(
                self.fbfd,
                x,
                y,
                (&mut px as *mut u32).cast::<std::ffi::c_void>(),
            )
        })
    }

    /// Fill a rectangle with a pre-packed pixel value. Same semantics as
    /// [`Fbink::fill_rect_gray`] (empty rect/`None` = full screen).
    pub fn fill_rect(
        &self,
        cfg: &Config,
        rect: Option<&Rect>,
        no_rota: bool,
        px: PackedPixel,
    ) -> Result<()> {
        let raw = rect.map(|r| r.to_raw());
        let raw_ptr = raw
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const sys::FBInkRect);
        let mut px = px.0;
        // Safety: cfg is a valid reference; raw_ptr is null or valid; px
        // points at a valid u32.
        check(unsafe {
            sys::fbink_fill_rect(
                self.fbfd,
                cfg.raw(),
                raw_ptr,
                no_rota,
                (&mut px as *mut u32).cast::<std::ffi::c_void>(),
            )
        })
    }

    /// Read back a single pixel as RGBA (grayscale fbs return `r == g == b`
    /// and `a == 0xFF`; BGR/565 swaps are handled for you).
    pub fn get_pixel(&self, x: u16, y: u16) -> Result<(u8, u8, u8, u8)> {
        let (mut r, mut g, mut b, mut a) = (0u8, 0u8, 0u8, 0u8);
        // Safety: all out-pointers are valid.
        let ret = unsafe { sys::fbink_get_pixel(self.fbfd, x, y, &mut r, &mut g, &mut b, &mut a) };
        check(ret)?;
        Ok((r, g, b, a))
    }

    /// Direct access to the framebuffer's backing buffer.
    ///
    /// # Safety
    ///
    /// * **Must not** be called before [`Fbink::init`], nor with a
    ///   [`Fbink::auto`] handle.
    /// * The returned pointer aliases the raw framebuffer: writing to it
    ///   bypasses *all* of FBInk's (and Rust's) safety checks. Its size may
    ///   change after a framebuffer state change (refresh via
    ///   [`Fbink::reinit`] and call again).
    ///
    /// Returns `None` on failure.
    pub unsafe fn fb_pointer(&self) -> Option<(*mut u8, usize)> {
        let mut size: usize = 0;
        // Safety: upheld by the caller; size is a valid out-pointer.
        let ptr = sys::fbink_get_fb_pointer(self.fbfd, &mut size);
        if ptr.is_null() {
            None
        } else {
            Some((ptr, size))
        }
    }

    /// A mostly untouched copy of the raw `linuxfb` structs.
    ///
    /// Prefer [`Fbink::state`], unless you really need the ioctl-level data.
    pub fn get_fb_info(&self) -> (sys::fb_var_screeninfo, sys::fb_fix_screeninfo) {
        let mut var_info: sys::fb_var_screeninfo = unsafe { std::mem::zeroed() };
        let mut fix_info: sys::fb_fix_screeninfo = unsafe { std::mem::zeroed() };
        // Safety: both out-pointers are valid.
        unsafe { sys::fbink_get_fb_info(&mut var_info, &mut fix_info) };
        (var_info, fix_info)
    }

    /// Set the framebuffer's native rotation and/or bitdepth.
    ///
    /// `rota` is a *native* linuxfb rotation value (see
    /// [`Fbink::rota_canonical_to_native`]); `bpp` is one of `4, 8, 16, 32`;
    /// `grayscale` is only meaningful at 8bpp on mxcfb/mtk. Each parameter
    /// accepts its `KEEP_CURRENT_*` magic constant to be left untouched,
    /// and [`TOGGLE_GRAYSCALE`] for `grayscale`.
    ///
    /// On success the state is reinitialized *now*: the return value behaves
    /// exactly like [`Fbink::reinit`]'s (only tested on Kobo & Kindle).
    pub fn set_fb_info(
        &self,
        rota: u32,
        bpp: u8,
        grayscale: u8,
        cfg: &Config,
    ) -> Result<ReinitChanges> {
        // Safety: cfg is a valid reference.
        let ret = unsafe { sys::fbink_set_fb_info(self.fbfd, rota, bpp, grayscale, cfg.raw()) };
        if ret < 0 {
            Err(Error::from_ret(ret))
        } else {
            Ok(ReinitChanges::from_raw(ret as u32))
        }
    }

    /// Toggle the sunxi "pen" refresh mode (Kobo Elipsa/Sage; only truly
    /// active with the [`crate::WaveformMode::A2`] &
    /// [`crate::WaveformMode::Du`] waveform modes).
    pub fn sunxi_toggle_ntx_pen_mode(&self, toggle: bool) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_sunxi_toggle_ntx_pen_mode(self.fbfd, toggle) })
    }

    /// Control at runtime how (re)inits handle rotation on sunxi, potentially
    /// overriding the accelerometer (Kobo Elipsa/Sage).
    ///
    /// On success the state is reinitialized *now*: the return value behaves
    /// exactly like [`Fbink::reinit`]'s.
    pub fn sunxi_ntx_enforce_rota(
        &self,
        mode: crate::SunxiForceRota,
        cfg: &Config,
    ) -> Result<ReinitChanges> {
        // Safety: cfg is a valid reference.
        let ret = unsafe { sys::fbink_sunxi_ntx_enforce_rota(self.fbfd, mode.to_raw(), cfg.raw()) };
        if ret < 0 {
            Err(Error::from_ret(ret))
        } else {
            Ok(ReinitChanges::from_raw(ret as u32))
        }
    }

    /// Setup the swipe animation direction & duration used by refreshes with
    /// `is_animated` set (Kindle MTK; `steps` max is 60 on Bellatrix).
    pub fn mtk_set_swipe_data(&self, direction: crate::MtkSwipeDirection, steps: u8) -> Result<()> {
        crate::mtk_set_swipe_data(direction, steps)
    }

    /// Set the screen regions to gray out with a checkered pattern
    /// (Kindle only).
    ///
    /// Both regions are *excluded* (i.e. *not* checkered); use the first
    /// element when only excluding one. An empty first region (or
    /// [`crate::MtkHalftone::Disabled`]) disables the feature. Does *not*
    /// refresh the screen.
    pub fn mtk_set_halftone(
        &self,
        exclude_regions: [Rect; 2],
        size: crate::MtkHalftone,
    ) -> Result<()> {
        let raw = [exclude_regions[0].to_raw(), exclude_regions[1].to_raw()];
        // Safety: raw is a valid 2-element array.
        check(unsafe { sys::fbink_mtk_set_halftone(self.fbfd, raw.as_ptr(), size.to_raw()) })
    }

    /// Toggle automatic REAGL upgrades for large enough refreshes
    /// (Kindle only).
    pub fn mtk_toggle_auto_reagl(&self, toggle: bool) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_mtk_toggle_auto_reagl(self.fbfd, toggle) })
    }

    /// Toggle the "pen" refresh mode (Kindle on Bellatrix3 only).
    ///
    /// Pen mode updates *must* use [`crate::WaveformMode::Du`] (or
    /// [`crate::WaveformMode::Dunm`] in nightmode): enforcing that is up to
    /// you.
    pub fn mtk_toggle_pen_mode(&self, toggle: bool) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_mtk_toggle_pen_mode(self.fbfd, toggle) })
    }

    /// Wait (up to 2s) for *every* pending refresh (Kindle MTK).
    ///
    /// Will always timeout while "fast mode" is active (i.e. after
    /// [`Fbink::mtk_toggle_auto_reagl`]`(false)`).
    pub fn wait_for_any_complete(&self) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_wait_for_any_complete(self.fbfd) })
    }

    /// Convert a "native" fb rotation value (from `vInfo.rotate`) to a
    /// "canonical" one (how the device is physically laid out). Kobo only.
    ///
    /// NOTE: failures are reported as *positive* errno values in the C API;
    /// they are mapped to [`Error::NotSupported`] & [`Error::OutOfRange`].
    pub fn rota_native_to_canonical(&self, rotate: u32) -> Result<u8> {
        rota_result(unsafe { sys::fbink_rota_native_to_canonical(rotate) } as i32).map(|v| v as u8)
    }

    /// Convert a "canonical" rotation value back to a "native" one. Kobo only.
    /// See [`Fbink::rota_native_to_canonical`] for the error handling.
    pub fn rota_canonical_to_native(&self, rotate: u8) -> Result<u32> {
        rota_result(unsafe { sys::fbink_rota_canonical_to_native(rotate) } as i32)
    }

    /// Scan the screen for Kobo's USBMS "Connect" button and optionally press
    /// it (Kobo i.MX only, deprecated).
    pub fn button_scan(&self, press_button: bool, nosleep: bool) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_button_scan(self.fbfd, press_button, nosleep) })
    }

    /// Wait for the end of a Kobo USBMS session, detecting a successful
    /// content import (Kobo i.MX only, deprecated).
    pub fn wait_for_usbms_processing(&self, force_unplug: bool) -> Result<()> {
        // Safety: plain call.
        check(unsafe { sys::fbink_wait_for_usbms_processing(self.fbfd, force_unplug) })
    }
}

impl Drop for Fbink {
    fn drop(&mut self) {
        if self.owned {
            // Safety: `owned` is true exactly while the fd is still open
            // (both `close()` and Drop clear it), so this closes it once and
            // only once.
            unsafe { sys::fbink_close(self.fbfd) };
        }
    }
}

/// A pixel pre-packed for the *current* framebuffer pixel format, as returned
/// by [`crate::pack_pixel_gray`]/[`crate::pack_pixel_rgba`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedPixel(pub u32);

/// Map the `rota_*` helpers' peculiar return convention: valid results are
/// `0-3`, errors are *positive* errno values.
fn rota_result(ret: i32) -> Result<u32> {
    match ret {
        x if x == ENOSYS => Err(Error::NotSupported),
        x if x == ERANGE => Err(Error::OutOfRange),
        v => Ok(v as u32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reinit_changes_flags() {
        let none = ReinitChanges::from_raw(0);
        assert!(none.is_empty());
        assert!(!none.bpp_changed());

        let all = ReinitChanges::from_raw(
            sys::OK_BPP_CHANGE
                | sys::OK_ROTA_CHANGE
                | sys::OK_LAYOUT_CHANGE
                | sys::OK_GRAYSCALE_CHANGE,
        );
        assert!(!all.is_empty());
        assert!(all.bpp_changed());
        assert!(all.rota_changed());
        assert!(all.layout_changed());
        assert!(all.grayscale_changed());
    }

    #[test]
    fn features_bitmasks() {
        let f = Features::DRAW | Features::IMAGE;
        assert!(f.contains(Features::DRAW));
        assert!(f.contains(Features::IMAGE));
        assert!(!f.contains(Features::DRAW | Features::IMAGE | Features::INPUT));
        assert!(Features::MINIMAL.is_empty());
        assert!(!Features::DRAW.is_empty());
    }

    #[test]
    fn rota_error_mapping() {
        assert_eq!(rota_result(2), Ok(2));
        assert_eq!(rota_result(38), Err(Error::NotSupported));
        assert_eq!(rota_result(34), Err(Error::OutOfRange));
    }
}
