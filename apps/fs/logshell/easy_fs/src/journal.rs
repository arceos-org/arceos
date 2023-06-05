use crate::block_cache::block_cache_manager;
use alloc::rc::Rc;
use core::cell::RefCell;

struct SystemWrapperInner {
    cache_manager: Rc<CacheManagerWrapper>,
    current_handle: Option<Rc<RefCell<jbd::Handle>>>,
}

pub struct SystemProvider(RefCell<SystemWrapperInner>);

impl SystemProvider {
    pub fn new() -> Self {
        Self(RefCell::new(SystemWrapperInner {
            cache_manager: Rc::new(CacheManagerWrapper),
            current_handle: None,
        }))
    }
}

impl jbd::sal::System for SystemProvider {
    fn get_buffer_provider(&self) -> Rc<dyn jbd::sal::BufferProvider> {
        self.0.borrow().cache_manager.clone()
    }
    fn get_current_handle(&self) -> Option<Rc<RefCell<jbd::Handle>>> {
        self.0.borrow().current_handle.as_ref().map(|x| x.clone())
    }
    fn get_time(&self) -> usize {
        0
    }
    fn set_current_handle(&self, handle: Option<Rc<RefCell<jbd::Handle>>>) {
        self.0.borrow_mut().current_handle = handle;
    }
}

pub struct CacheManagerWrapper;

impl jbd::sal::BufferProvider for CacheManagerWrapper {
    fn get_buffer(
        &self,
        dev: &Rc<dyn jbd::sal::BlockDevice>,
        block_id: usize,
    ) -> Option<Rc<dyn jbd::sal::Buffer>> {
        get_buffer_dyn(dev, block_id)
    }
}

pub fn get_buffer_dyn(
    dev: &Rc<dyn jbd::sal::BlockDevice>,
    block_id: usize,
) -> Option<Rc<dyn jbd::sal::Buffer>> {
    Some(block_cache_manager().get_block_cache(block_id, dev.clone()))
}
