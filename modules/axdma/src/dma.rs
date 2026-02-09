use core::{alloc::Layout, ptr::NonNull};

use axalloc::{UsageKind, global_allocator};
use axallocator::{AllocError, AllocResult};
use axhal::mem::virt_to_phys;
use crate_interface::call_interface;
use kspin::SpinNoIrq;
use log::error;
use memory_addr::{PAGE_SIZE_4K, VirtAddr, va};
use page_table_multiarch::MappingFlags;

use crate::{BusAddr, DMAInfo, phys_to_bus};

pub(crate) static ALLOCATOR: SpinNoIrq<DmaAllocator> = SpinNoIrq::new(DmaAllocator::new());

pub(crate) struct DmaAllocator {}

impl DmaAllocator {
    pub const fn new() -> Self {
        Self {}
    }

    /// Allocate arbitrary number of bytes. Returns the left bound of the
    /// allocated region.
    ///
    /// It firstly tries to allocate from the coherent byte allocator. If there is no
    /// memory, it asks the global page allocator for more memory and adds it to the
    /// byte allocator.
    pub unsafe fn alloc_coherent(&mut self, layout: Layout) -> AllocResult<DMAInfo> {
        let num_pages = layout_pages(&layout);

        let addr = global_allocator()
            .alloc_dma32_pages(num_pages, PAGE_SIZE_4K, UsageKind::Dma)
            .map_err(|_e| AllocError::NoMemory)?;

        let vaddr = va!(addr);

        self.update_flags(
            vaddr,
            num_pages,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::UNCACHED,
        )?;

        Ok(DMAInfo {
            cpu_addr: unsafe { NonNull::new_unchecked(addr as *mut u8) },
            bus_addr: virt_to_bus(vaddr),
        })
    }

    fn update_flags(
        &mut self,
        vaddr: VirtAddr,
        num_pages: usize,
        flags: MappingFlags,
    ) -> AllocResult<()> {
        let size = num_pages * PAGE_SIZE_4K;
        call_interface!(crate::DmaProtectIf::protect_memory(vaddr, size, flags)).map_err(|e| {
            error!("change table flag fail: {e:?}");
            AllocError::NoMemory
        })
    }

    /// Gives back the allocated region to the byte allocator.
    pub unsafe fn dealloc_coherent(&mut self, dma: DMAInfo, layout: Layout) {
        let num_pages = layout_pages(&layout);
        global_allocator().dealloc_pages(dma.cpu_addr.as_ptr() as usize, num_pages, UsageKind::Dma);
    }
}

fn virt_to_bus(addr: VirtAddr) -> BusAddr {
    let paddr = virt_to_phys(addr);
    phys_to_bus(paddr)
}

const fn layout_pages(layout: &Layout) -> usize {
    memory_addr::align_up_4k(layout.size()) / PAGE_SIZE_4K
}
