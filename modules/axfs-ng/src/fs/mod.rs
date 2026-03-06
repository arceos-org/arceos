use axdriver::AxBlockDevice;
use axfs_ng_vfs::{Filesystem, VfsResult};

cfg_if::cfg_if! {
    if #[cfg(feature = "ext4")] {
        mod ext4;
        type DefaultFilesystem = ext4::Ext4Filesystem;
    } else if #[cfg(feature = "fat")] {
        mod fat;
        type DefaultFilesystem = fat::FatFilesystem;
    } else {
        struct DefaultFilesystem;
        impl DefaultFilesystem {
            pub fn new(_dev: AxBlockDevice) -> VfsResult<Filesystem> {
                panic!("No filesystem feature enabled");
            }
        }
    }
}

pub fn new_default(dev: AxBlockDevice) -> VfsResult<Filesystem> {
    DefaultFilesystem::new(dev)
}
