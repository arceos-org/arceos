use dma_api::Impl;

use crate::{
    arch::cache::{dcache_flush_range, dcache_invalidate_range},
    mem::virt_to_phys,
};

struct DmaImpl;

impl Impl for DmaImpl {
    fn map(addr: core::ptr::NonNull<u8>, _size: usize, _direction: dma_api::Direction) -> u64 {
        let phys = virt_to_phys((addr.as_ptr() as usize).into()).as_usize();
        phys as u64
    }

    fn unmap(_addr: core::ptr::NonNull<u8>, _size: usize) {}

    fn flush(addr: core::ptr::NonNull<u8>, size: usize) {
        dcache_flush_range(addr, size);
    }

    fn invalidate(addr: core::ptr::NonNull<u8>, size: usize) {
        dcache_invalidate_range(addr, size);
    }
}

dma_api::set_impl!(DmaImpl);
