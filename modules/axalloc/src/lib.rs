//! [ArceOS](https://github.com/arceos-org/arceos) global memory allocator.
//!
//! It provides [`GlobalAllocator`], which implements the trait
//! [`core::alloc::GlobalAlloc`]. A static global variable of type
//! [`GlobalAllocator`] is defined with the `#[global_allocator]` attribute, to
//! be registered as the standard library’s default allocator.

#![no_std]

extern crate alloc;

#[doc(no_inline)]
pub use os_memory::{BootState, MemRegion, MemRegionFlags, global_allocator};



mod page;

pub fn name() -> &'static str {
    os_memory::allocator_name()
}

pub fn init<B: BootState>() {
    os_memory::init_allocator::<B>();
}
