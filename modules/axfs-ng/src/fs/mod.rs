#[cfg(feature = "fat")]
pub mod fat;

#[cfg(feature = "ext4")]
pub mod ext4;

use axdriver::AxBlockDevice;
use axfs_ng_vfs::{Filesystem, VfsResult};
use cfg_if::cfg_if;
use lock_api::RawMutex;

pub fn new_default<M: RawMutex + Send + Sync + 'static>(
    dev: AxBlockDevice,
) -> VfsResult<Filesystem<M>> {
    cfg_if! {
        if #[cfg(feature = "ext4")] {
            ext4::Ext4Filesystem::new(dev)
        } else if #[cfg(feature = "fat")] {
            Ok(fat::FatFilesystem::new(dev))
        } else {
            panic!("No filesystem feature enabled");
        }
    }
}
