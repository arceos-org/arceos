//! syscall number: all syscall constants
//! most of them have the same meaning as UNIX syscall
#![no_std]
#![allow(missing_docs)]

pub const SYS_EXIT: usize = 10;
pub const SYS_SPAWN: usize = 11;
pub const SYS_YIELD: usize = 12;
pub const SYS_SLEEP: usize = 13;
pub const SYS_TIME_NANO: usize = 14;
pub const SYS_SBRK: usize = 20;
pub const SYS_FUTEX: usize = 30;
pub const SYS_FORK: usize = 40;
pub const SYS_WAIT: usize = 41;

// ============
// The following are from [redox](https://gitlab.redox-os.org/redox-os/syscall/-/blob/master/src/number.rs),
// in order to identify types of parameter.

pub const SYS_CLASS: usize = 0xF000_0000;
pub const SYS_CLASS_PATH: usize = 0x1000_0000;
pub const SYS_CLASS_FILE: usize = 0x2000_0000;

pub const SYS_ARG: usize = 0x0F00_0000;
pub const SYS_ARG_SLICE: usize = 0x0100_0000;
pub const SYS_ARG_MSLICE: usize = 0x0200_0000;
pub const SYS_ARG_PATH: usize = 0x0300_0000;

pub const SYS_RET: usize = 0x00F0_0000;
pub const SYS_RET_FILE: usize = 0x0010_0000;

pub const SYS_LINK: usize = SYS_CLASS_PATH | SYS_ARG_PATH | 9;
/// open: path ptr, path len, option
pub const SYS_OPEN: usize = SYS_CLASS_PATH | SYS_RET_FILE | 5;
pub const SYS_RMDIR: usize = SYS_CLASS_PATH | 84;
pub const SYS_UNLINK: usize = SYS_CLASS_PATH | 10;

pub const SYS_CLOSE: usize = SYS_CLASS_FILE | 6;
pub const SYS_DUP: usize = SYS_CLASS_FILE | SYS_RET_FILE | 41;
pub const SYS_DUP2: usize = SYS_CLASS_FILE | SYS_RET_FILE | 63;
pub const SYS_READ: usize = SYS_CLASS_FILE | SYS_ARG_MSLICE | 3;
pub const SYS_WRITE: usize = SYS_CLASS_FILE | SYS_ARG_SLICE | 4;
pub const SYS_LSEEK: usize = SYS_CLASS_FILE | 19;
pub const SYS_FCHMOD: usize = SYS_CLASS_FILE | 94;
pub const SYS_FCHOWN: usize = SYS_CLASS_FILE | 207;
pub const SYS_FCNTL: usize = SYS_CLASS_FILE | 55;
pub const SYS_FEVENT: usize = SYS_CLASS_FILE | 927;
pub const SYS_FMAP_OLD: usize = SYS_CLASS_FILE | SYS_ARG_SLICE | 90;
pub const SYS_FMAP: usize = SYS_CLASS_FILE | SYS_ARG_SLICE | 900;
pub const SYS_FUNMAP_OLD: usize = SYS_CLASS_FILE | 91;
pub const SYS_FUNMAP: usize = SYS_CLASS_FILE | 92;
pub const SYS_FPATH: usize = SYS_CLASS_FILE | SYS_ARG_MSLICE | 928;
pub const SYS_FRENAME: usize = SYS_CLASS_FILE | SYS_ARG_PATH | 38;
pub const SYS_FSTAT: usize = SYS_CLASS_FILE | SYS_ARG_MSLICE | 28;
pub const SYS_FSTATVFS: usize = SYS_CLASS_FILE | SYS_ARG_MSLICE | 100;
pub const SYS_FSYNC: usize = SYS_CLASS_FILE | 118;
pub const SYS_FTRUNCATE: usize = SYS_CLASS_FILE | 93;
pub const SYS_FUTIMENS: usize = SYS_CLASS_FILE | SYS_ARG_SLICE | 320;

// ===========

// We do not use features, because these constants will cause little side effects.
pub mod futex;
pub mod io;
