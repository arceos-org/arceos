//! Scheme: Concepts derived from [Redox](https://redox-os.org/).
//! This crate is originally from <https://gitlab.redox-os.org/redox-os/syscall/>, with some simplification and modification.
#![no_std]

/// Packet: data sent between scheme implementations
/// a wrapper of IO syscalls
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Packet {
    /// syscall id
    pub id: u64,
    /// process id (not used)
    pub pid: usize,
    /// user id (not used)
    pub uid: u32,
    /// group id (not used)
    pub gid: u32,
    /// param 1
    pub a: usize,
    /// param 2
    pub b: usize,
    /// param 3
    pub c: usize,
    /// param 4
    pub d: usize,
}

impl Deref for Packet {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Packet as *const u8,
                core::mem::size_of::<Packet>(),
            )
        }
    }
}

impl DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut Packet as *mut u8,
                core::mem::size_of::<Packet>(),
            )
        }
    }
}

/// return type of `stat` syscall, NOT in UNIX format
pub type Stat = axfs_vfs::VfsNodeAttr;

unsafe fn str_from_raw_parts(ptr: *const u8, len: usize) -> Option<&'static str> {
    let slice = core::slice::from_raw_parts(ptr, len);
    core::str::from_utf8(slice).ok()
}

mod scheme;
use core::ops::{Deref, DerefMut};

pub use crate::scheme::Scheme;
