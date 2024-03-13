cfg_if::cfg_if! {
    if #[cfg(feature = "myfs")] {
        pub mod myfs;
        /// The block size of the file system.
        pub const BLOCK_SIZE: usize = 512;
    } else if #[cfg(feature = "fatfs")] {
        pub mod fatfs;
        pub use fatfs::BLOCK_SIZE;
    } else if #[cfg(feature = "ext4fs")] {
        pub mod ext4fs;
        pub use ext4fs::BLOCK_SIZE;
    }
}

#[cfg(feature = "devfs")]
pub use axfs_devfs as devfs;

#[cfg(feature = "ramfs")]
pub use axfs_ramfs as ramfs;
