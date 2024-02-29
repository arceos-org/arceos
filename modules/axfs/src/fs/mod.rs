cfg_if::cfg_if! {
    if #[cfg(feature = "myfs")] {
        pub mod myfs;
    } else if #[cfg(feature = "fatfs")] {
        pub mod fatfs;
        pub use fatfs::BLOCK_SIZE as FAT_BLOCK_SIZE;
    }
}

#[cfg(feature = "devfs")]
pub use axfs_devfs as devfs;

#[cfg(feature = "ramfs")]
pub use axfs_ramfs as ramfs;
