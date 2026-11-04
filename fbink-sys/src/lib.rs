//! Raw FFI bindings to [FBInk](https://github.com/NiLuJe/FBInk)
//! ("FrameBuffer eInker"), a library to print text & images to an eInk Linux
//! framebuffer (Kobo, Kindle, Cervantes, reMarkable, PocketBook...).
//!
//! This is the `-sys` crate: everything here is `unsafe`, thinly documented and
//! maps 1:1 to FBInk's C API (see FBInk's own `fbink.h` for the authoritative
//! documentation). For a safe, idiomatic API, use the [`fbink`](https://docs.rs/fbink)
//! crate instead.
//!
//! # Versioning
//!
//! The bindings are generated at build time from a *pinned* FBInk revision
//! (see `build.rs`), together with the C sources compiled into the static
//! library: the two can therefore never get out of sync.
//!
//! # Build requirements
//!
//! * A C compiler (via `cc`) and `libclang` (via `bindgen`).
//! * `curl` or `wget` to fetch the pinned FBInk sources (or set
//!   `FBINK_SOURCE_DIR` to a local `git clone --recursive` of FBInk).
//! * `FBINK_TARGET` set to the platform you're building for
//!   (`linux`, `kobo`, `kindle`, `kindle-legacy`, `cervantes`, `remarkable`
//!   or `pocketbook`); defaults to `linux`.
//!
//! # License
//!
//! FBInk is licensed under the [GPL-3.0-or-later](https://github.com/NiLuJe/FBInk/blob/master/LICENSE);
//! statically linking it (as this crate does) makes the resulting binary subject
//! to the same terms.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    /// Smoke test: the static library links and the global entry points resolve.
    #[test]
    fn version_and_target_are_linked() {
        let version = unsafe { CStr::from_ptr(fbink_version()) };
        assert!(!version.to_bytes().is_empty());
        println!(
            "FBInk version: {}, target: {}",
            version.to_string_lossy(),
            unsafe { fbink_target() }
        );
    }
}
