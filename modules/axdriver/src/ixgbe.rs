use axalloc::global_allocator;
use axdriver_net::ixgbe::{IxgbeHal, PhysAddr as IxgbePhysAddr};
use axhal::mem::{alloc_coherent, dealloc_coherent, phys_to_virt, virt_to_phys};
use core::{alloc::Layout, ptr::NonNull};

pub struct IxgbeHalImpl;

unsafe impl IxgbeHal for IxgbeHalImpl {
    fn dma_alloc(size: usize) -> (IxgbePhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(size, 8).unwrap();

        unsafe {
            if let Some(dma) = alloc_coherent(layout) {
                (
                    dma.bus_addr as usize,
                    NonNull::new_unchecked(dma.cpu_addr as _),
                )
            } else {
                (0, NonNull::dangling())
            }
        }
    }

    unsafe fn dma_dealloc(paddr: IxgbePhysAddr, vaddr: NonNull<u8>, size: usize) -> i32 {
        let layout = Layout::from_size_align(size, 8).unwrap();

        unsafe {
            dealloc_coherent(
                DMAInfo {
                    cpu_addr: vaddr.as_ptr() as _,
                    bus_addr: paddr as _,
                },
                Layout::from_size_align(pages * 0x1000, 0x1000).unwrap(),
            );
        }
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
