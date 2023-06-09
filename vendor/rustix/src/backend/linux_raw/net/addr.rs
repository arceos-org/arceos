//! Socket address utilities.
//!
//! # Safety
//!
//! This file casts between `&[c_char]` used in C APIs and `&[u8]` used in Rust
//! APIs.
#![allow(unsafe_code)]

use super::super::c;
use crate::ffi::CStr;
use crate::{io, path};
use core::cmp::Ordering;
use core::convert::TryInto;
use core::hash::{Hash, Hasher};
use core::{fmt, slice};

/// `struct sockaddr_un`
#[derive(Clone)]
#[doc(alias = "sockaddr_un")]
pub struct SocketAddrUnix {
    pub(crate) unix: c::sockaddr_un,
    len: c::socklen_t,
}

impl SocketAddrUnix {
    /// Construct a new Unix-domain address from a filesystem path.
    #[inline]
    pub fn new<P: path::Arg>(path: P) -> io::Result<Self> {
        path.into_with_c_str(Self::_new)
    }

    #[inline]
    fn _new(path: &CStr) -> io::Result<Self> {
        let mut unix = Self::init();
        let bytes = path.to_bytes_with_nul();
        if bytes.len() > unix.sun_path.len() {
            return Err(io::Errno::NAMETOOLONG);
        }
        for (i, b) in bytes.iter().enumerate() {
            unix.sun_path[i] = *b as c::c_char;
        }
        let len = offsetof_sun_path() + bytes.len();
        let len = len.try_into().unwrap();
        Ok(Self { unix, len })
    }

    /// Construct a new abstract Unix-domain address from a byte slice.
    #[inline]
    pub fn new_abstract_name(name: &[u8]) -> io::Result<Self> {
        let mut unix = Self::init();
        if 1 + name.len() > unix.sun_path.len() {
            return Err(io::Errno::NAMETOOLONG);
        }
        unix.sun_path[0] = b'\0' as c::c_char;
        for (i, b) in name.iter().enumerate() {
            unix.sun_path[1 + i] = *b as c::c_char;
        }
        let len = offsetof_sun_path() + 1 + name.len();
        let len = len.try_into().unwrap();
        Ok(Self { unix, len })
    }

    fn init() -> c::sockaddr_un {
        c::sockaddr_un {
            sun_family: c::AF_UNIX as _,
            sun_path: [0; 108],
        }
    }

    /// For a filesystem path address, return the path.
    #[inline]
    pub fn path(&self) -> Option<&CStr> {
        let len = self.len();
        if len != 0 && self.unix.sun_path[0] != b'\0' as c::c_char {
            let end = len as usize - offsetof_sun_path();
            let bytes = &self.unix.sun_path[..end];
            // SAFETY: `from_raw_parts` to convert from `&[c_char]` to `&[u8]`.
            // And `from_bytes_with_nul_unchecked` since the string is
            // NUL-terminated.
            unsafe {
                Some(CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(
                    bytes.as_ptr().cast(),
                    bytes.len(),
                )))
            }
        } else {
            None
        }
    }

    /// For an abstract address, return the identifier.
    #[inline]
    pub fn abstract_name(&self) -> Option<&[u8]> {
        let len = self.len();
        if len != 0 && self.unix.sun_path[0] == b'\0' as c::c_char {
            let end = len as usize - offsetof_sun_path();
            let bytes = &self.unix.sun_path[1..end];
            // SAFETY: `from_raw_parts` to convert from `&[c_char]` to `&[u8]`.
            unsafe { Some(slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len())) }
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn addr_len(&self) -> c::socklen_t {
        self.len
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.addr_len() as usize
    }
}

impl PartialEq for SocketAddrUnix {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].eq(&other.unix.sun_path[..other_len])
    }
}

impl Eq for SocketAddrUnix {}

impl PartialOrd for SocketAddrUnix {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].partial_cmp(&other.unix.sun_path[..other_len])
    }
}

impl Ord for SocketAddrUnix {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        let self_len = self.len() - offsetof_sun_path();
        let other_len = other.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].cmp(&other.unix.sun_path[..other_len])
    }
}

impl Hash for SocketAddrUnix {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_len = self.len() - offsetof_sun_path();
        self.unix.sun_path[..self_len].hash(state)
    }
}

impl fmt::Debug for SocketAddrUnix {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            path.fmt(fmt)
        } else if let Some(name) = self.abstract_name() {
            name.fmt(fmt)
        } else {
            "(unnamed)".fmt(fmt)
        }
    }
}

/// `struct sockaddr_storage` as a raw struct.
pub type SocketAddrStorage = c::sockaddr;

/// Return the offset of the `sun_path` field of `sockaddr_un`.
#[inline]
pub(crate) fn offsetof_sun_path() -> usize {
    let z = c::sockaddr_un {
        sun_family: 0_u16,
        sun_path: [0; 108],
    };
    (crate::utils::as_ptr(&z.sun_path) as usize) - (crate::utils::as_ptr(&z) as usize)
}
