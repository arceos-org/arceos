pub(crate) use libc::*;

/// `PROC_SUPER_MAGIC`—The magic number for the procfs filesystem.
#[cfg(all(any(target_os = "android", target_os = "linux"), target_env = "musl"))]
pub(crate) const PROC_SUPER_MAGIC: u32 = 0x0000_9fa0;

/// `NFS_SUPER_MAGIC`—The magic number for the NFS filesystem.
#[cfg(all(any(target_os = "android", target_os = "linux"), target_env = "musl"))]
pub(crate) const NFS_SUPER_MAGIC: u32 = 0x0000_6969;
