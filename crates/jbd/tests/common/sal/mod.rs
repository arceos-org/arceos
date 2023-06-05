use std::{cell::RefCell, io, rc::Rc};

use self::{cache::BlockCacheManager, dev::FileDevice};
use jbd::{
    sal::{BlockDevice, BufferProvider, System},
    Handle,
};

pub mod cache;
pub mod dev;

struct UserSystemInner {
    device: Rc<dyn BlockDevice>,
    cache_manager: Rc<BlockCacheManager>,
    current_handle: Option<Rc<RefCell<Handle>>>,
}

pub struct UserSystem {
    inner: RefCell<UserSystemInner>,
}

impl UserSystem {
    pub fn new(path: &str, nblocks: usize) -> Result<Self, io::Error> {
        let device = FileDevice::new(path, nblocks)?;
        let cache_manager = Rc::new(BlockCacheManager::new());
        Ok(Self {
            inner: RefCell::new(UserSystemInner {
                device: Rc::new(device),
                cache_manager,
                current_handle: None,
            }),
        })
    }

    pub fn block_device(&self) -> Rc<dyn BlockDevice> {
        self.inner.borrow().device.clone()
    }
}

impl System for UserSystem {
    fn get_buffer_provider(&self) -> Rc<dyn BufferProvider> {
        self.inner.borrow().cache_manager.clone()
    }
    fn get_time(&self) -> usize {
        // TODO
        0
    }
    fn get_current_handle(&self) -> Option<Rc<RefCell<Handle>>> {
        self.inner.borrow().current_handle.clone()
    }
    fn set_current_handle(&self, handle: Option<Rc<RefCell<Handle>>>) {
        self.inner.borrow_mut().current_handle = handle;
    }
}
