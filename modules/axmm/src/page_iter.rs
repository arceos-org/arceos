//! Memory Page Iterator Wrapper Module
//!
//! Provides a unified iteration interface across different page sizes,
//! supporting address iteration for 4K, 2M, and 1G page sizes.
//! The design is inspired by the Iterator Wrapper pattern,
//! using an enum to unify the behavior of iterators for different page sizes.

use axhal::{mem::VirtAddr, paging::PageSize};
use memory_addr::PageIter;
/// 4K page size constant (4,096 bytes) and iterator type alias
pub use memory_addr::PageIter4K;

/// 2MB page size constant (2,097,152 bytes)
pub const PAGE_SIZE_2M: usize = 0x20_0000;

/// 2MB page iterator type alias
///
/// Wraps the `PageIter` struct with a fixed page size of `PAGE_SIZE_2M`
pub type PageIter2M<A> = PageIter<PAGE_SIZE_2M, A>;

/// 1GB page size constant (1,073,741,824 bytes)
pub const PAGE_SIZE_1G: usize = 0x4000_0000;

/// 1GB page iterator type alias
///
/// Wraps the `PageIter` struct with a fixed page size of `PAGE_SIZE_1G`
pub type PageIter1G<A> = PageIter<PAGE_SIZE_1G, A>;

/// Page Iterator Wrapper Enum
///
/// Unifies the iterator interfaces for different page sizes, enabling
/// transparent access to address iteration.
///
/// The design follows the Iterator Wrapper pattern, eliminating type
/// differences between iterators of varying page sizes.
pub enum PageIterWrapper {
    /// 4K page iterator variant
    Size4K(PageIter4K<VirtAddr>),
    /// 2M page iterator variant
    Size2M(PageIter2M<VirtAddr>),
    /// 1G page iterator variant
    Size1G(PageIter1G<VirtAddr>),
}

impl PageIterWrapper {
    /// Creates an iterator wrapper instance for the specified page size
    ///
    /// # Parameters
    /// - `start`: Starting virtual address (inclusive), which must be aligned
    ///   to the `page_size`
    /// - `end`: Ending virtual address (exclusive), which must also be aligned
    ///   to the `page_size`
    /// - `page_size`: Enum type specifying the page size
    ///
    /// # Returns
    /// Returns an `Option` wrapping the iterator instance. Returns `None` if
    /// the page size is unsupported.
    ///
    /// # Example
    /// ```rust
    /// use axhal::{mem::VirtAddr, paging::PageSize};
    /// use axmm::page_iter_wrapper::PageIterWrapper;
    /// let start_addr = VirtAddr::from(0x1000);
    /// let end_addr = VirtAddr::from(0x3000);
    /// let iter = PageIterWrapper::new(start_addr, end_addr, PageSize::Size4K);
    /// ```
    pub fn new(start: VirtAddr, end: VirtAddr, page_size: PageSize) -> Option<Self> {
        match page_size {
            PageSize::Size4K => PageIter4K::<VirtAddr>::new(start, end).map(Self::Size4K),
            PageSize::Size2M => PageIter2M::<VirtAddr>::new(start, end).map(Self::Size2M),
            PageSize::Size1G => PageIter1G::<VirtAddr>::new(start, end).map(Self::Size1G),
        }
    }
}

/// Iterator trait implementation
///
/// Unifies address iteration behavior for all three page sizes,
/// providing a transparent external access interface.
/// The implementation follows the paginated iterator design pattern,
/// using an enum to dispatch calls to the underlying iterators.
impl Iterator for PageIterWrapper {
    type Item = VirtAddr;

    /// Retrieves the next virtual address
    ///
    /// # Returns
    /// Returns an `Option` wrapping the virtual address. Returns `None` when
    /// the iteration is complete.
    ///
    /// # Implementation Details
    /// Based on the current enum variant, the corresponding underlying iterator
    /// is called. The original behavior of each page size iterator is
    /// preserved.
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Size4K(iter) => iter.next(),
            Self::Size2M(iter) => iter.next(),
            Self::Size1G(iter) => iter.next(),
        }
    }
}
