//!
//! An ELF file loader.
//! It takes an elf file as an input, parses all segments, and output its address info, binary data, and access flags.
//!
#![no_std]

#[macro_use]
extern crate alloc;

use memory_addr::VirtAddr;
use page_table_entry::MappingFlags;

/// Segments of the elf file
pub struct SegmentEntry<'a> {
    /// Start address
    pub start_addr: VirtAddr,
    /// Segment length
    pub size: usize,
    /// Segment data
    pub data: &'a [u8],
    /// Access Flags (RWX)
    pub flags: MappingFlags,
}

impl<'a> SegmentEntry<'a> {
    /// creates the `SegmentEntry` from binary data.   
    pub fn new(data: &'a [u8]) -> Option<alloc::vec::Vec<SegmentEntry<'a>>> {
        let elf = xmas_elf::ElfFile::new(data).ok()?;
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        if magic != [0x7f, 0x45, 0x4c, 0x46] {
            return None;
        }
        let ph_count = elf_header.pt2.ph_count();
        let mut result = vec![];
        for i in 0..ph_count {
            let ph = elf.program_header(i).ok()?;
            if ph.get_type().ok()? == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let size: usize = ph.mem_size() as usize;
                let mut flags = MappingFlags::empty();
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    flags |= MappingFlags::READ;
                }
                if ph_flags.is_write() {
                    flags |= MappingFlags::WRITE;
                }
                if ph_flags.is_execute() {
                    flags |= MappingFlags::EXECUTE;
                }
                result.push(SegmentEntry {
                    start_addr: start_va,
                    size,
                    data: &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize],
                    flags,
                });
            }
        }
        Some(result)
    }
}
