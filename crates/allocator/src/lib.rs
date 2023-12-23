//! Various allocator algorithms in a unified interface.
//!
//! There are three types of allocators:
//!
//! - [`ByteAllocator`]: Byte-granularity memory allocator. (e.g.,
//!   [`BuddyByteAllocator`], [`SlabByteAllocator`])
//! - [`PageAllocator`]: Page-granularity memory allocator. (e.g.,
//!   [`BitmapPageAllocator`])
//! - [`IdAllocator`]: Used to allocate unique IDs.

#![no_std]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

#[cfg(feature = "bitmap")]
mod bitmap;
#[cfg(feature = "bitmap")]
pub use bitmap::BitmapPageAllocator;

#[cfg(feature = "buddy")]
mod buddy;
#[cfg(feature = "buddy")]
pub use buddy::BuddyByteAllocator;

#[cfg(feature = "slab")]
mod slab;
#[cfg(feature = "slab")]
pub use slab::SlabByteAllocator;

#[cfg(feature = "tlsf")]
mod tlsf;
#[cfg(feature = "tlsf")]
pub use tlsf::TlsfByteAllocator;

use core::alloc::Layout;
use core::ptr::NonNull;

/// The error type used for allocation.
#[derive(Debug)]
pub enum AllocError {
    /// Invalid `size` or `align_pow2`. (e.g. unaligned)
    InvalidParam,
    /// Memory added by `add_memory` overlapped with existed memory.
    MemoryOverlap,
    /// No enough memory to allocate.
    NoMemory,
    /// Deallocate an unallocated memory region.
    NotAllocated,
}

/// A [`Result`] type with [`AllocError`] as the error type.
pub type AllocResult<T = ()> = Result<T, AllocError>;

/// The base allocator inherited by other allocators.
pub trait BaseAllocator {
    /// Initialize the allocator with a free memory region.
    fn init(&mut self, start: usize, size: usize);

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult;
}

/// Byte-granularity allocator.
pub trait ByteAllocator: BaseAllocator {
    /// Allocate memory with the given size (in bytes) and alignment.
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>>;

    /// Deallocate memory at the given position, size, and alignment.
    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout);

    /// Returns total memory size in bytes.
    fn total_bytes(&self) -> usize;

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize;

    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize;
}

/// Page-granularity allocator.
pub trait PageAllocator: BaseAllocator {
    /// The size of a memory page.
    const PAGE_SIZE: usize;

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize>;

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize);

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize;

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize;

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize;
}

/// Used to allocate unique IDs (e.g., thread ID).
pub trait IdAllocator: BaseAllocator {
    /// Allocate contiguous IDs with given count and alignment.
    fn alloc_id(&mut self, count: usize, align_pow2: usize) -> AllocResult<usize>;

    /// Deallocate contiguous IDs with given position and count.
    fn dealloc_id(&mut self, start_id: usize, count: usize);

    /// Whether the given `id` was allocated.
    fn is_allocated(&self, id: usize) -> bool;

    /// Mark the given `id` has been allocated and cannot be reallocated.
    fn alloc_fixed_id(&mut self, id: usize) -> AllocResult;

    /// Returns the maximum number of supported IDs.
    fn size(&self) -> usize;

    /// Returns the number of allocated IDs.
    fn used(&self) -> usize;

    /// Returns the number of available IDs.
    fn available(&self) -> usize;
}

#[inline]
const fn align_down(pos: usize, align: usize) -> usize {
    pos & !(align - 1)
}

#[inline]
const fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}

#[cfg(feature = "allocator_api")]
mod allocator_api {
    extern crate alloc;

    use super::ByteAllocator;
    use alloc::rc::Rc;
    use core::alloc::{AllocError, Allocator, Layout};
    use core::cell::RefCell;
    use core::ptr::NonNull;

    /// A byte-allocator wrapped in [`Rc<RefCell>`] that implements [`core::alloc::Allocator`].
    pub struct AllocatorRc<A: ByteAllocator>(Rc<RefCell<A>>);

    impl<A: ByteAllocator> AllocatorRc<A> {
        /// Creates a new allocator with the given memory pool.
        pub fn new(mut inner: A, pool: &mut [u8]) -> Self {
            inner.init(pool.as_mut_ptr() as usize, pool.len());
            Self(Rc::new(RefCell::new(inner)))
        }
    }

    unsafe impl<A: ByteAllocator> Allocator for AllocatorRc<A> {
        fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
            match layout.size() {
                0 => Ok(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
                size => {
                    let raw_addr = self.0.borrow_mut().alloc(layout).map_err(|_| AllocError)?;
                    Ok(NonNull::slice_from_raw_parts(raw_addr, size))
                }
            }
        }

        unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
            self.0.borrow_mut().dealloc(ptr, layout)
        }
    }

    impl<A: ByteAllocator> Clone for AllocatorRc<A> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
}

#[cfg(feature = "allocator_api")]
pub use allocator_api::AllocatorRc;
