//! Error handling: mapping of FBInk's negative return codes to [`Error`].
//!
//! FBInk reports failures as *negative* errno values (e.g. `-EINVAL`), with
//! `-(EXIT_FAILURE)` (i.e. `-1`) as the generic fallback. A few calls
//! ([`crate::Fbink::rota_native_to_canonical`] & friends) instead return
//! *positive* errno values on failure, because their signature returns an
//! unsigned integer; those are mapped through [`Error::from_errno`].

use std::ffi::{CString, NulError};
use std::fmt;

/// The `Result` type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

// Linux (asm-generic) errno values, hardcoded to avoid a `libc` dependency.
// All the targets FBInk (and thus this crate) supports are Linux on ARM or x86,
// which all use these values.
pub(crate) const ENODEV: i32 = 19;
pub(crate) const EINVAL: i32 = 22;
pub(crate) const ENOSPC: i32 = 28;
pub(crate) const ERANGE: i32 = 34;
pub(crate) const ENOSYS: i32 = 38;
pub(crate) const ENODATA: i32 = 61;
pub(crate) const EILSEQ: i32 = 84;
pub(crate) const ENOTSUP: i32 = 95; // == EOPNOTSUPP on Linux

/// An error reported by FBInk.
///
/// The variants map the negative return codes documented in FBInk's `fbink.h`;
/// on unknown codes, [`Error::Other`] carries the *positive* errno value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Generic failure: `-(EXIT_FAILURE)`.
    ///
    /// NOTE: FBInk uses `-(EXIT_FAILURE)` (i.e. `-1`) as its default failure
    /// code, which makes it ambiguous with `-EPERM`; it is special-cased first.
    Failure,
    /// `-ENOSYS` or `-ENOTSUP`: the call is not supported in this build
    /// (e.g. MINIMAL build w/o DRAW) or on this platform/device.
    NotSupported,
    /// `-EINVAL`: invalid argument (e.g. an empty string).
    Invalid,
    /// `-EILSEQ`: the string was not a valid UTF-8 sequence.
    InvalidUtf8,
    /// `-ERANGE`: value out of range (e.g. OT margins, invalid rotation).
    OutOfRange,
    /// `-ENOSPC`: the content doesn't fit (e.g. `no_truncation` was requested).
    NoSpace,
    /// `-ENODATA`: required data is missing (e.g. no OT font loaded).
    NoData,
    /// `-ENODEV`: the device is missing or the call happened out of order
    /// (e.g. before `fbink_init`).
    NoDevice,
    /// A Rust string handed to FBInk contained an interior NUL byte.
    ///
    /// This is a purely Rust-side error: such a string cannot be represented as
    /// a C string, so the call was never made.
    InteriorNul,
    /// Any other errno, stored as its *positive* value (e.g. `Other(125)` for
    /// `-ECANCELED`).
    Other(i32),
}

impl Error {
    /// Map a (negative) return value from an FBInk call to an [`Error`].
    pub(crate) fn from_ret(ret: i32) -> Self {
        match ret {
            -1 => Error::Failure,
            _ if ret == -ENOSYS || ret == -ENOTSUP => Error::NotSupported,
            _ if ret == -EINVAL => Error::Invalid,
            _ if ret == -EILSEQ => Error::InvalidUtf8,
            _ if ret == -ERANGE => Error::OutOfRange,
            _ if ret == -ENOSPC => Error::NoSpace,
            _ if ret == -ENODATA => Error::NoData,
            _ if ret == -ENODEV => Error::NoDevice,
            _ => Error::Other(-ret),
        }
    }

    /// Map a *positive* errno value to an [`Error`]
    /// (e.g. from `fbink_open`'s errno, or the `rota_*` helpers' return value).
    pub(crate) fn from_errno(errno: i32) -> Self {
        if errno == 0 {
            Error::Failure
        } else {
            Error::from_ret(-errno)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Failure => write!(f, "generic FBInk failure (-(EXIT_FAILURE))"),
            Error::NotSupported => {
                write!(
                    f,
                    "not supported by this FBInk build or platform (ENOSYS/ENOTSUP)"
                )
            }
            Error::Invalid => write!(f, "invalid argument (EINVAL)"),
            Error::InvalidUtf8 => write!(f, "string is not valid UTF-8 (EILSEQ)"),
            Error::OutOfRange => write!(f, "value out of range (ERANGE)"),
            Error::NoSpace => write!(f, "content does not fit (ENOSPC)"),
            Error::NoData => write!(f, "required data missing (ENODATA)"),
            Error::NoDevice => write!(f, "device missing or call out of order (ENODEV)"),
            Error::InteriorNul => write!(f, "string contains an interior NUL byte"),
            Error::Other(e) => write!(f, "FBInk failed with errno {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<NulError> for Error {
    fn from(_: NulError) -> Self {
        Error::InteriorNul
    }
}

/// Require a non-negative return value, mapping it to `Ok(())`.
pub(crate) fn check(ret: i32) -> Result<()> {
    if ret >= 0 {
        Ok(())
    } else {
        Err(Error::from_ret(ret))
    }
}

/// Require a non-negative return value, yielding it as a `u32`
/// (e.g. `fbink_print`'s amount of lines printed).
pub(crate) fn check_nonneg(ret: i32) -> Result<u32> {
    if ret >= 0 {
        Ok(ret as u32)
    } else {
        Err(Error::from_ret(ret))
    }
}

/// Convert a string to a C string, mapping interior NULs to
/// [`Error::InteriorNul`].
pub(crate) fn cstring(s: &str) -> Result<CString> {
    Ok(CString::new(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_negative_return_codes() {
        assert_eq!(Error::from_ret(-1), Error::Failure);
        assert_eq!(Error::from_ret(-ENOSYS), Error::NotSupported);
        assert_eq!(Error::from_ret(-ENOTSUP), Error::NotSupported);
        assert_eq!(Error::from_ret(-EINVAL), Error::Invalid);
        assert_eq!(Error::from_ret(-EILSEQ), Error::InvalidUtf8);
        assert_eq!(Error::from_ret(-ERANGE), Error::OutOfRange);
        assert_eq!(Error::from_ret(-ENOSPC), Error::NoSpace);
        assert_eq!(Error::from_ret(-ENODATA), Error::NoData);
        assert_eq!(Error::from_ret(-ENODEV), Error::NoDevice);
        assert_eq!(Error::from_ret(-125), Error::Other(125));
    }

    #[test]
    fn maps_positive_errnos() {
        assert_eq!(Error::from_errno(ENOSYS), Error::NotSupported);
        assert_eq!(Error::from_errno(ERANGE), Error::OutOfRange);
        assert_eq!(Error::from_errno(2), Error::Other(2));
        // -1 is ambiguous: never maps to Other.
        assert_eq!(Error::from_errno(1), Error::Failure);
    }

    #[test]
    fn checks_return_values() {
        assert_eq!(check(0), Ok(()));
        assert_eq!(check(-EINVAL), Err(Error::Invalid));
        assert_eq!(check_nonneg(3), Ok(3));
        assert_eq!(check_nonneg(-1), Err(Error::Failure));
    }

    #[test]
    fn rejects_interior_nul() {
        let err = cstring("foo\0bar").unwrap_err();
        assert_eq!(err, Error::InteriorNul);
    }
}
