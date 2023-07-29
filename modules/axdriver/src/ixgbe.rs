use axalloc::global_allocator;
use axhal::mem::{phys_to_virt, virt_to_phys};
use core::{alloc::Layout, ptr::NonNull};
use driver_net::ixgbe::{IxgbeHal, PhysAddr as IxgbePhysAddr};

pub struct IxgbeHalImpl;

unsafe impl IxgbeHal for IxgbeHalImpl {
    fn dma_alloc(size: usize) -> (IxgbePhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let vaddr = if let Ok(vaddr) = global_allocator().alloc(layout) {
            vaddr.get()
        } else {
            return (0, NonNull::dangling());
        };
        let paddr = virt_to_phys(vaddr.into());
        let ptr = NonNull::new(vaddr as _).unwrap();
        (paddr.as_usize(), ptr)
    }

    unsafe fn dma_dealloc(_paddr: IxgbePhysAddr, vaddr: NonNull<u8>, size: usize) -> i32 {
        let layout = Layout::from_size_align(size, 8).unwrap();
        global_allocator().dealloc(vaddr.addr(), layout);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: IxgbePhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(phys_to_virt(paddr.into()).as_mut_ptr()).unwrap()
    }

    unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> IxgbePhysAddr {
        virt_to_phys((vaddr.as_ptr() as usize).into()).into()
    }

    fn wait_until(duration: core::time::Duration) -> Result<(), &'static str> {
        axhal::time::busy_wait_until(duration);
        Ok(())
    }
}
