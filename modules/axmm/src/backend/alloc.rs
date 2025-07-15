use axhal::paging::{MappingFlags, PageSize, PageTable, PagingError, PagingResult};
use memory_addr::{PageIter4K, VirtAddr};

use super::{alloc_frame, dealloc_frame};

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
    ) -> PagingResult {
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
                let frame = alloc_frame(true).ok_or(PagingError::NoMemory)?;
                pt.map(addr, frame, PageSize::Size4K, flags)?;
            }
        }
        Ok(())
    }

    pub(crate) fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> PagingResult {
        debug!("Alloc::unmap [{:#x}, {:#x})", start, start + size);
        for addr in PageIter4K::new(start, start + size).unwrap() {
            if let Ok((frame, _)) = pt.unmap(addr) {
                // Deallocate the physical frame if there is a mapping in the page table.
                dealloc_frame(frame);
            } else {
                // Deallocation is needn't if the page is not mapped.
            }
        }
        Ok(())
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
                    let frame = alloc_frame(true).ok_or(PagingError::NoMemory)?;
                    // Allocate a physical frame lazily and map it to the fault address.
                    // `vaddr` does not need to be aligned. It will be automatically
                    // aligned during `pt.remap` regardless of the page size.
                    pt.map(addr, frame, PageSize::Size4K, flags)?;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
