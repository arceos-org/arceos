use core::cell::{RefCell, UnsafeCell};

use crate::dev::Disk;
use alloc::sync::Arc;
use axfs_vfs::{VfsNodeRef, VfsOps};
use easy_fs::{self, Inode, Mutex, BLOCK_SZ};

struct DiskWrapper(RefCell<Disk>);

unsafe impl Sync for DiskWrapper {}
unsafe impl Send for DiskWrapper {}

impl easy_fs::BlockDevice for DiskWrapper {
    fn read_block(&self, block: usize, buf: &mut [u8]) {
        let mut disk = self.0.borrow_mut();
        disk.set_position((block * BLOCK_SZ) as u64);
        _ = disk.read_one(buf);
    }

    fn write_block(&self, block: usize, buf: &[u8]) {
        let mut disk = self.0.borrow_mut();
        disk.set_position((block * BLOCK_SZ) as u64);
        _ = disk.write_one(buf);
    }
}

pub struct EasyFileSystem {
    inner: Arc<Mutex<easy_fs::EasyFileSystem>>,
    root_dir: UnsafeCell<Option<Arc<Inode>>>,
}

unsafe impl Sync for EasyFileSystem {}
unsafe impl Send for EasyFileSystem {}

impl EasyFileSystem {
    pub fn new(disk: Disk) -> Self {
        let wrapper = Arc::new(DiskWrapper(RefCell::new(disk)));
        let inner = easy_fs::EasyFileSystem::create(wrapper, 16 * 2048, 1);
        Self {
            inner,
            root_dir: UnsafeCell::new(None),
        }
    }

    pub fn init(&'static self) {
        // must be called before later operations
        unsafe {
            *self.root_dir.get() = Some(Arc::new(easy_fs::EasyFileSystem::root_inode(
                &self.inner.clone(),
            )));
        }
    }
}

impl VfsOps for EasyFileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        let root_dir = unsafe { (*self.root_dir.get()).as_ref().unwrap() };
        root_dir.clone()
    }
}
