use axalloc::global_allocator;
use axhal::mem::{phys_to_virt, virt_to_phys};
use axhal::paging::{MappingFlags, PageSize, PageTable};
use memory_addr::{PAGE_SIZE_4K, PageIter4K, PhysAddr, VirtAddr};

use super::Backend;

fn alloc_frame(zeroed: bool) -> Option<PhysAddr> {
    let vaddr = VirtAddr::from(global_allocator().alloc_pages(1, PAGE_SIZE_4K).ok()?);
    if zeroed {
        unsafe { core::ptr::write_bytes(vaddr.as_mut_ptr(), 0, PAGE_SIZE_4K) };
    }
    let paddr = virt_to_phys(vaddr);
    Some(paddr)
}

fn dealloc_frame(frame: PhysAddr) {
    let vaddr = phys_to_virt(frame);
    global_allocator().dealloc_pages(vaddr.as_usize(), 1);
}

impl Backend {
    /// Creates a new allocation mapping backend.
    pub const fn new_alloc(populate: bool) -> Self {
        Self::Alloc { populate }
    }

    pub(crate) fn map_alloc(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
        populate: bool,
    ) -> bool {
        debug!(
            "map_alloc: [{:#x}, {:#x}) {:?} (populate={})",
            start,
            start + size,
            flags,
            populate
        );
        if populate {
            // allocate all possible physical frames for populated mapping.
            let mut cursor = pt.cursor();
            for addr in PageIter4K::new(start, start + size).unwrap() {
                if let Some(frame) = alloc_frame(true)
                    && cursor.map(addr, frame, PageSize::Size4K, flags).is_err()
                {
                    // Mapping failed; roll back any previously mapped pages in this range
                    // and deallocate their frames to avoid leaks and partial mappings.
                    for rollback_addr in PageIter4K::new(start, addr).unwrap() {
                        if let Ok((mapped_frame, _, page_size)) = cursor.unmap(rollback_addr) {
                            // We only expect 4K pages here, but avoid touching huge pages.
                            if !page_size.is_huge() {
                                dealloc_frame(mapped_frame);
                            }
                        }
                    }
                    // Deallocate the just-allocated frame that failed to map.
                    dealloc_frame(frame);
                    return false;
                }
            }
            true
        } else {
            // Map to a empty entry for on-demand mapping.
            let flags = MappingFlags::empty();
            pt.cursor()
                .map_region(start, |_| 0.into(), size, flags, false)
                .is_ok()
        }
    }

    pub(crate) fn unmap_alloc(
        &self,
        start: VirtAddr,
        size: usize,
        pt: &mut PageTable,
        _populate: bool,
    ) -> bool {
        debug!("unmap_alloc: [{:#x}, {:#x})", start, start + size);
        for addr in PageIter4K::new(start, start + size).unwrap() {
            if let Ok((frame, _, page_size)) = pt.cursor().unmap(addr) {
                // Deallocate the physical frame if there is a mapping in the
                // page table.
                if page_size.is_huge() {
                    return false;
                }
                dealloc_frame(frame);
            }
        }
        true
    }

    pub(crate) fn handle_page_fault_alloc(
        &self,
        vaddr: VirtAddr,
        orig_flags: MappingFlags,
        pt: &mut PageTable,
        populate: bool,
    ) -> bool {
        if populate {
            false // Populated mappings should not trigger page faults.
        } else if let Some(frame) = alloc_frame(true) {
            // Allocate a physical frame lazily and map it to the fault address.
            // `vaddr` does not need to be aligned. It will be automatically
            // aligned during `pt.remap` regardless of the page size.
            let res = pt.cursor().remap(vaddr, frame, orig_flags);
            if let Err(e) = &res {
                debug!(
                    "handle_page_fault_alloc: remap failed for {:#x}: {:?}",
                    vaddr, e
                );
                dealloc_frame(frame);
            }
            res.is_ok()
        } else {
            false
        }
    }
}
