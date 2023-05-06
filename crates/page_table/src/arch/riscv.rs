//! RISC-V specific page table structures.

use crate::{PageTable64, PagingMetaData};
use page_table_entry::riscv::Rv64PTE;

/// Metadata of RISC-V Sv39 page tables.
#[derive(Clone, Copy)]
pub struct Sv39MetaData;

/// Metadata of RISC-V Sv48 page tables.
#[derive(Clone, Copy)]
pub struct Sv48MetaData;

impl const PagingMetaData for Sv39MetaData {
    const LEVELS: usize = 3;
    const PA_MAX_BITS: usize = 56;
    const VA_MAX_BITS: usize = 39;
}

impl const PagingMetaData for Sv48MetaData {
    const LEVELS: usize = 4;
    const PA_MAX_BITS: usize = 56;
    const VA_MAX_BITS: usize = 48;
}

/// Sv39: Page-Based 39-bit (3 levels) Virtual-Memory System.
pub type Sv39PageTable<I> = PageTable64<Sv39MetaData, Rv64PTE, I>;

/// Sv48: Page-Based 48-bit (4 levels) Virtual-Memory System.
pub type Sv48PageTable<I> = PageTable64<Sv48MetaData, Rv64PTE, I>;
