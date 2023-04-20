//! AArch64 specific page table structures.

use crate::{PageTable64, PagingMetaData};
use page_table_entry::aarch64::A64PTE;

/// Metadata of AArch64 page tables.
#[derive(Copy, Clone)]
pub struct A64PagingMetaData;

impl const PagingMetaData for A64PagingMetaData {
    const LEVELS: usize = 4;
    const PA_MAX_BITS: usize = 48;
    const VA_MAX_BITS: usize = 48;

    fn vaddr_is_valid(vaddr: usize) -> bool {
        let top_bits = vaddr >> Self::VA_MAX_BITS;
        top_bits == 0 || top_bits == 0xffff
    }
}

/// AArch64 VMSAv8-64 translation table.
pub type A64PageTable<I> = PageTable64<A64PagingMetaData, A64PTE, I>;
