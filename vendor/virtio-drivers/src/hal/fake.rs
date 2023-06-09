//! Fake HAL implementation for tests.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};
use alloc::alloc::{alloc_zeroed, dealloc, handle_alloc_error};
use core::{alloc::Layout, ptr::NonNull};

#[derive(Debug)]
pub struct FakeHal;

/// Fake HAL implementation for use in unit tests.
unsafe impl Hal for FakeHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        assert_ne!(pages, 0);
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        // Safe because the size and alignment of the layout are non-zero.
        let ptr = unsafe { alloc_zeroed(layout) };
        if let Some(ptr) = NonNull::new(ptr) {
            (ptr.as_ptr() as PhysAddr, ptr)
        } else {
            handle_alloc_error(layout);
        }
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        assert_ne!(pages, 0);
        let layout = Layout::from_size_align(pages * PAGE_SIZE, PAGE_SIZE).unwrap();
        // Safe because the layout is the same as was used when the memory was allocated by
        // `dma_alloc` above.
        unsafe {
            dealloc(vaddr.as_ptr(), layout);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        // Nothing to do, as the host already has access to all memory.
        virt_to_phys(vaddr)
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }
}

fn virt_to_phys(vaddr: usize) -> PhysAddr {
    vaddr
}
