use alloc::{sync::Arc, vec::Vec};
use core::ops::Deref;

use axhal::paging::{MappingFlags, PageSize, PageTable};
use memory_addr::{PAGE_SIZE_4K, PhysAddr, VirtAddr};

use super::{alloc_frame, dealloc_frame};

pub struct SharedPages(Vec<PhysAddr>);

impl SharedPages {
    pub fn new(n: usize) -> Option<Arc<Self>> {
        // Deallocate frames if allocation fails.
        let mut pages = SharedPages(Vec::with_capacity(n));
        for _ in 0..n {
            pages.0.push(alloc_frame(true)?);
        }
        Some(Arc::new(pages))
    }
}

impl Deref for SharedPages {
    type Target = [PhysAddr];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for SharedPages {
    fn drop(&mut self) {
        for &frame in &self.0 {
            dealloc_frame(frame);
        }
    }
}

// FIXME: This implementation does not allow map or unmap partial ranges.
#[derive(Clone)]
pub struct Shared {
    pub(crate) pages: Arc<SharedPages>,
}

impl Shared {
    pub(crate) fn map(&self, start: VirtAddr, flags: MappingFlags, pt: &mut PageTable) -> bool {
        debug!(
            "Shared::map [{:#x}, {:#x}) {:?}",
            start,
            start + self.pages.len() * PAGE_SIZE_4K,
            flags
        );
        // allocate all possible physical frames for populated mapping.
        for (i, frame) in self.pages.iter().enumerate() {
            let addr = start + i * PAGE_SIZE_4K;
            if let Ok(tlb) = pt.map(addr, *frame, PageSize::Size4K, flags) {
                tlb.flush();
            } else {
                return false;
            }
        }
        true
    }

    pub(crate) fn unmap(&self, start: VirtAddr, pt: &mut PageTable) -> bool {
        debug!(
            "Shared::unmap [{:#x}, {:#x})",
            start,
            start + self.pages.len() * PAGE_SIZE_4K
        );
        for i in 0..self.pages.len() {
            let addr = start + i * PAGE_SIZE_4K;
            if let Ok((_, page_size, tlb)) = pt.unmap(addr) {
                // Deallocate the physical frame if there is a mapping in the
                // page table.
                if page_size.is_huge() {
                    return false;
                }
                tlb.flush();
            } else {
                // Deallocation is needn't if the page is not mapped.
            }
        }
        true
    }
}
