use core::ops::DerefMut;

use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};
use rand::{rngs::SmallRng, Fill, SeedableRng};
use spin::Mutex;

/// A random device behaves like `/dev/random` or `/dev/urandom`.
///
/// It always returns a chunk of random bytes when read, and all writes are discarded.
///
/// TODO: update entropy pool with data written.
pub struct RandomDev(Mutex<SmallRng>);

impl Default for RandomDev {
    fn default() -> Self {
        Self(Mutex::new(SmallRng::from_seed([0; 32])))
    }
}

impl VfsNodeOps for RandomDev {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        buf.try_fill(self.0.lock().deref_mut()).unwrap();
        Ok(buf.len())
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(buf.len())
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        Ok(())
    }

    axfs_vfs::impl_vfs_non_dir_default! {}
}
