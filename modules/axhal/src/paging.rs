//! Page table manipulation.

use axalloc::*;

use crate::mem::{phys_to_virt, virt_to_phys,  PhysAddr, VirtAddr, PAGE_SIZE_4K};

#[doc(no_inline)]
pub use page_table_multiarch::{MappingFlags, PageSize, PagingError, PagingResult};



