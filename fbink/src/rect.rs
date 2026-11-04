//! [`Rect`], the safe wrapper around FBInk's `FBInkRect`.
//!
//! NOTE: unlike an mxcfb rectangle, `left` (x) comes *before* `top` (y).

use fbink_sys as sys;

/// A rectangle in absolute framebuffer coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Rect {
    /// x
    pub left: u16,
    /// y
    pub top: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(left: u16, top: u16, width: u16, height: u16) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    /// The empty rectangle `{0, 0, 0, 0}`.
    pub const fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// An empty rectangle has a zero width and/or height.
    ///
    /// FBInk interprets empty rects as "the full screen" in most APIs.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// The raw `fbink-sys` rectangle.
    pub const fn to_raw(self) -> sys::FBInkRect {
        sys::FBInkRect {
            left: self.left,
            top: self.top,
            width: self.width,
            height: self.height,
        }
    }

    /// Wrap a raw `fbink-sys` rectangle.
    pub const fn from_raw(raw: sys::FBInkRect) -> Self {
        Self::new(raw.left, raw.top, raw.width, raw.height)
    }
}

impl From<Rect> for sys::FBInkRect {
    fn from(rect: Rect) -> Self {
        rect.to_raw()
    }
}

impl From<sys::FBInkRect> for Rect {
    fn from(raw: sys::FBInkRect) -> Self {
        Rect::from_raw(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emptiness() {
        assert!(Rect::empty().is_empty());
        assert!(Rect::new(0, 0, 0, 10).is_empty());
        assert!(Rect::new(0, 0, 10, 0).is_empty());
        assert!(!Rect::new(1, 2, 3, 4).is_empty());
    }

    #[test]
    fn raw_roundtrip() {
        let rect = Rect::new(10, 20, 30, 40);
        let raw: sys::FBInkRect = rect.into();
        assert_eq!(raw.left, 10);
        assert_eq!(raw.top, 20);
        assert_eq!(Rect::from(raw), rect);
    }
}
