use axhal::mem::virt_to_phys;
use dma_api::Osal;

pub fn setup_dma_api() {
    dma_api::init(&OsImpl);
}

struct OsImpl;

impl Osal for OsImpl {
    fn map(
        &self,
        addr: core::ptr::NonNull<u8>,
        _size: usize,
        _direction: dma_api::Direction,
    ) -> u64 {
        virt_to_phys((addr.as_ptr() as usize).into()).as_usize() as _
    }

    fn unmap(&self, _addr: core::ptr::NonNull<u8>, _size: usize) {}
}
