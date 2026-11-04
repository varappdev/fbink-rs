//! RAII framebuffer dumps: [`Dump`] wraps FBInk's `FBInkDump`, which comes
//! with an infamous double-free trap (a dirty struct is *recycled* — i.e. its
//! data freed — by the next dump call). [`Dump`]'s [`Drop`] takes care of the
//! matching `fbink_free_dump_data()` call exactly once.

use crate::config::Config;
use crate::error::{Result, check};
use crate::fb::Fbink;
use crate::rect::Rect;
use fbink_sys as sys;

/// A framebuffer snapshot, as taken by [`Fbink::dump`],
/// [`Fbink::region_dump`] or [`Fbink::rect_dump`], restorable via
/// [`Fbink::restore`].
///
/// The pixel storage is allocated by FBInk and freed when the `Dump` is
/// dropped (or by the next dump call into it, which this wrapper prevents).
#[derive(Debug)]
pub struct Dump {
    raw: sys::FBInkDump,
}

impl Dump {
    /// An empty dump, as expected by FBInk before the first use.
    pub fn new() -> Self {
        // FBInkDump is a POD C struct: all-zero is the required initial state.
        Self {
            raw: unsafe { std::mem::zeroed() },
        }
    }

    /// The raw pixels, as dumped from the framebuffer.
    ///
    /// On most devices (the exceptions being 4bpp & 16bpp fbs) this is valid
    /// input for [`Fbink::print_raw_data`].
    pub fn data(&self) -> &[u8] {
        if self.raw.data.is_null() || self.raw.size == 0 {
            &[]
        } else {
            // Safety: FBInk allocated `size` bytes at `data`, and this wrapper
            // owns the allocation until Drop.
            unsafe { std::slice::from_raw_parts(self.raw.data, self.raw.size) }
        }
    }

    /// The screen area this dump covers.
    pub fn area(&self) -> Rect {
        Rect::from_raw(self.raw.area)
    }

    /// The restore clip rectangle (absolute screen coordinates).
    ///
    /// Only the intersection of `clip` and [`Dump::area`] will be restored;
    /// it must intersect the area. Empty (the default) means "restore
    /// everything".
    pub fn clip(&self) -> Rect {
        Rect::from_raw(self.raw.clip)
    }

    /// Restrict [`Fbink::restore`] to a rectangular area of the screen
    /// (absolute coordinates, must intersect [`Dump::area`]).
    pub fn set_clip(&mut self, clip: Rect) {
        self.raw.clip = clip.to_raw();
    }

    /// Whether this is a full-screen dump.
    pub fn is_full(&self) -> bool {
        self.raw.is_full
    }

    /// Mark the dump as full-screen: needs to be flipped by hand before
    /// clipping (i.e. [`Dump::set_clip`]) a full dump.
    pub fn set_is_full(&mut self, is_full: bool) {
        self.raw.is_full = is_full;
    }

    /// The native rotation the dump was taken at.
    pub fn rota(&self) -> u8 {
        self.raw.rota
    }

    /// The bitdepth the dump was taken at.
    pub fn bpp(&self) -> u8 {
        self.raw.bpp
    }

    /// Scanline length in bytes (padding included).
    pub fn stride(&self) -> usize {
        self.raw.stride
    }

    /// The raw `fbink-sys` dump struct.
    pub fn raw(&self) -> &sys::FBInkDump {
        &self.raw
    }

    pub(crate) fn raw_mut(&mut self) -> &mut sys::FBInkDump {
        &mut self.raw
    }
}

impl Default for Dump {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Dump {
    fn drop(&mut self) {
        if !self.raw.data.is_null() {
            // Safety: data is non-null (i.e. allocated by FBInk and not yet
            // freed); this resets it to NULL internally.
            unsafe { sys::fbink_free_dump_data(&mut self.raw) };
        }
    }
}

impl Fbink {
    /// Dump the full screen.
    pub fn dump(&self) -> Result<Dump> {
        let mut dump = Dump::new();
        // Safety: dump's raw struct is a valid, zeroed FBInkDump.
        check(unsafe { sys::fbink_dump(self.fd(), dump.raw_mut()) })?;
        Ok(dump)
    }

    /// Dump a region of the screen (offsets honor negative values).
    ///
    /// Positioning honors any combination of `halign`/`valign`, `row`/`col` &
    /// `x_off`/`y_off` from `cfg`.
    pub fn region_dump(
        &self,
        x_off: i16,
        y_off: i16,
        w: u16,
        h: u16,
        cfg: &Config,
    ) -> Result<Dump> {
        let mut dump = Dump::new();
        // Safety: see Fbink::dump.
        check(unsafe {
            sys::fbink_region_dump(self.fd(), x_off, y_off, w, h, cfg.raw(), dump.raw_mut())
        })?;
        Ok(dump)
    }

    /// Dump the area described by `rect`, used *as is* (no rotation or
    /// positioning trickery): ideal for a [`crate::get_last_rect`] result.
    /// `None` or an empty rect dumps the full screen.
    pub fn rect_dump(&self, rect: Option<&Rect>) -> Result<Dump> {
        let raw = rect.map(|r| r.to_raw());
        let raw_ptr = raw
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const sys::FBInkRect);
        let mut dump = Dump::new();
        // Safety: raw_ptr is null or valid; see Fbink::dump for the dump.
        check(unsafe { sys::fbink_rect_dump(self.fd(), raw_ptr, dump.raw_mut()) })?;
        Ok(dump)
    }

    /// Restore a dump taken at the *current* bitdepth & rotation (call
    /// [`Fbink::reinit`] first to make sure they still match): the dump's
    /// pixels are put back exactly where they were captured.
    ///
    /// Fails with [`crate::Error::NotSupported`] when the dump doesn't match
    /// the current fb state (or is out of bounds), [`crate::Error::Invalid`]
    /// when there is no data to restore. Does *not* free the dump.
    pub fn restore(&self, cfg: &Config, dump: &Dump) -> Result<()> {
        // Safety: both references are valid.
        check(unsafe { sys::fbink_restore(self.fd(), cfg.raw(), dump.raw()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_dump_has_no_data() {
        let dump = Dump::new();
        assert!(dump.data().is_empty());
        assert!(dump.area().is_empty());
        assert!(dump.clip().is_empty());
        assert!(!dump.is_full());
        assert_eq!(dump.bpp(), 0);
    }

    #[test]
    fn clip_and_full_are_user_editable() {
        let mut dump = Dump::new();
        dump.set_clip(Rect::new(1, 2, 3, 4));
        assert_eq!(dump.clip(), Rect::new(1, 2, 3, 4));
        dump.set_is_full(true);
        assert!(dump.is_full());
    }
}
