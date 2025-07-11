use axalloc::global_allocator;
use axhal::{
    mem::{phys_to_virt, virt_to_phys},
    paging::{MappingFlags, PageSize, PageTable, PagingError, PagingResult},
};
use memory_addr::{PAGE_SIZE_4K, PageIter4K, PhysAddr, VirtAddr};

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

/// Allocation mapping backend.
///
/// If `populate` is `true`, all physical frames are allocated when the
/// mapping is created, and no page faults are triggered during the memory
/// access. Otherwise, the physical frames are allocated on demand (by
/// handling page faults).
#[derive(Clone)]
pub struct Alloc {
    populate: bool,
}

impl Alloc {
    /// Creates a new allocation mapping backend.
    pub(crate) const fn new(populate: bool) -> Self {
        Self { populate }
    }

    pub(crate) fn map(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> bool {
        debug!(
            "Alloc::map [{:#x}, {:#x}) {:?} (populate={})",
            start,
            start + size,
            flags,
            self.populate
        );
        if self.populate {
            // allocate all possible physical frames for populated mapping.
            for addr in PageIter4K::new(start, start + size).unwrap() {
                if let Some(frame) = alloc_frame(true) {
                    if let Ok(tlb) = pt.map(addr, frame, PageSize::Size4K, flags) {
                        tlb.ignore(); // TLB flush on map is unnecessary, as there are no outdated mappings.
                    } else {
                        return false;
                    }
                }
            }
        } else {
            // create mapping entries on demand later in
            // `handle_page_fault`.
        }
        true
    }

    pub(crate) fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> bool {
        debug!("Alloc::unmap [{:#x}, {:#x})", start, start + size);
        for addr in PageIter4K::new(start, start + size).unwrap() {
            if let Ok((frame, page_size, tlb)) = pt.unmap(addr) {
                // Deallocate the physical frame if there is a mapping in the
                // page table.
                if page_size.is_huge() {
                    return false;
                }
                tlb.flush();
                dealloc_frame(frame);
            } else {
                // Deallocation is needn't if the page is not mapped.
            }
        }
        true
    }

    pub(crate) fn populate(
        &self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        pt: &mut PageTable,
    ) -> PagingResult {
        if self.populate {
            return Err(PagingError::AlreadyMapped);
        }
        for addr in PageIter4K::new(start, start + size).unwrap() {
            match pt.query(addr) {
                Ok(_) => {}
                Err(PagingError::NotMapped) => {
                    if let Some(frame) = alloc_frame(true) {
                        // Allocate a physical frame lazily and map it to the fault address.
                        // `vaddr` does not need to be aligned. It will be automatically
                        // aligned during `pt.remap` regardless of the page size.
                        pt.map(addr, frame, PageSize::Size4K, flags)
                            .map(|tlb| tlb.flush())?;
                    } else {
                        return Err(PagingError::NoMemory);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
