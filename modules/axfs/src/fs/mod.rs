#[cfg(feature = "fatfs")]
pub mod fatfs;

#[cfg(feature = "devfs")]
pub use axfs_devfs as devfs;

#[cfg(feature = "easyfs")]
pub mod easyfs;
