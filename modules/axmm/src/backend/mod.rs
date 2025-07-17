//! Memory mapping backends.
use allocator::AllocError;
use axalloc::{UsageKind, global_allocator};
use axerrno::{LinuxError, LinuxResult};
use axhal::{
    mem::{phys_to_virt, virt_to_phys},
    paging::{MappingFlags, PageSize, PageTable, PagingError},
};
use enum_dispatch::enum_dispatch;
use memory_addr::{PAGE_SIZE_4K, PhysAddr, VirtAddr, VirtAddrRange};
use memory_set::MappingBackend;

pub mod alloc;
pub mod linear;
pub mod shared;

pub use shared::SharedPages;

use crate::{page_info::frame_table, page_iter::PageIterWrapper};

fn alloc_to_linux_error(err: AllocError) -> LinuxError {
    warn!("Allocation error: {:?}", err);
    match err {
        AllocError::NoMemory => LinuxError::ENOMEM,
        _ => LinuxError::EINVAL,
    }
}

fn alloc_frame(zeroed: bool, size: PageSize) -> LinuxResult<PhysAddr> {
    let page_size = size as usize;
    let num_pages = page_size / PAGE_SIZE_4K;
    let vaddr = VirtAddr::from(
        global_allocator()
            .alloc_pages(num_pages, page_size, UsageKind::UserMem)
            .map_err(alloc_to_linux_error)?,
    );
    if zeroed {
        unsafe { core::ptr::write_bytes(vaddr.as_mut_ptr(), 0, page_size) };
    }
    let paddr = virt_to_phys(vaddr);

    frame_table().inc_ref(paddr);

    Ok(paddr)
}

fn dealloc_frame(frame: PhysAddr, align: PageSize) {
    if frame_table().dec_ref(frame) > 1 {
        return;
    }

    let vaddr = phys_to_virt(frame);
    let page_size: usize = align.into();
    let num_pages = page_size / PAGE_SIZE_4K;
    global_allocator().dealloc_pages(vaddr.as_usize(), num_pages, UsageKind::UserMem);
}

fn paging_to_linux_error(err: PagingError) -> LinuxError {
    warn!("Paging error: {:?}", err);
    match err {
        PagingError::NoMemory => LinuxError::ENOMEM,
        _ => LinuxError::EINVAL,
    }
}

fn pages_in(range: VirtAddrRange, align: PageSize) -> LinuxResult<PageIterWrapper> {
    PageIterWrapper::new(range.start, range.end, align).ok_or(LinuxError::EINVAL)
}

#[enum_dispatch]
pub trait BackendOps {
    /// Returns the page size of the backend.
    fn page_size(&self) -> PageSize;

    /// Map a memory region.
    fn map(&self, range: VirtAddrRange, flags: MappingFlags, pt: &mut PageTable)
    -> LinuxResult<()>;

    /// Unmap a memory region.
    fn unmap(&self, range: VirtAddrRange, pt: &mut PageTable) -> LinuxResult<()>;

    /// Populate a memory region. Returns number of pages populated.
    fn populate(
        &self,
        _range: VirtAddrRange,
        _flags: MappingFlags,
        _access_flags: MappingFlags,
        _pt: &mut PageTable,
    ) -> LinuxResult<usize> {
        Ok(0)
    }

    /// Duplicates this mapping for use in a different page table.
    ///
    /// This differs from `clone`, which is designed for splitting a mapping
    /// within the same table.
    ///
    /// [`BackendOps::map`] will be latter called to the returned backend.
    fn clone_map(
        &self,
        range: VirtAddrRange,
        flags: MappingFlags,
        old_pt: &mut PageTable,
        new_pt: &mut PageTable,
    ) -> LinuxResult<Backend>;
}

/// A unified enum type for different memory mapping backends.
#[derive(Clone)]
#[enum_dispatch(BackendOps)]
pub enum Backend {
    Linear(linear::LinearBackend),
    Alloc(alloc::AllocBackend),
    Shared(shared::SharedBackend),
}

impl MappingBackend for Backend {
    type Addr = VirtAddr;
    type Flags = MappingFlags;
    type PageTable = PageTable;

    fn map(&self, start: VirtAddr, size: usize, flags: MappingFlags, pt: &mut PageTable) -> bool {
        let range = VirtAddrRange::from_start_size(start, size);
        if let Err(err) = BackendOps::map(self, range, flags, pt) {
            warn!("Failed to map area: {:?}", err);
            false
        } else {
            true
        }
    }

    fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> bool {
        let range = VirtAddrRange::from_start_size(start, size);
        if let Err(err) = BackendOps::unmap(self, range, pt) {
            warn!("Failed to unmap area: {:?}", err);
            false
        } else {
            true
        }
    }

    fn protect(
        &self,
        start: Self::Addr,
        size: usize,
        new_flags: Self::Flags,
        pt: &mut Self::PageTable,
    ) -> bool {
        pt.protect_region(start, size, new_flags).is_ok()
    }
}
