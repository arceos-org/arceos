#![no_std]
#![feature(extract_if)]
#![feature(btree_extract_if)]

mod area;
mod backend;
pub use area::MapArea;
use axerrno::{AxError, AxResult};
pub use backend::MemBackend;

extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec};
use core::{mem::size_of, ptr::copy_nonoverlapping};
use page_table_entry::GenericPTE;
#[macro_use]
extern crate log;

use axhal::{
    mem::{memory_regions, phys_to_virt, PhysAddr, VirtAddr, PAGE_SIZE_4K},
    paging::{MappingFlags, PageSize, PageTable},
};
use xmas_elf::symbol_table::Entry;

pub(crate) const REL_GOT: u32 = 6;
pub(crate) const REL_PLT: u32 = 7;
pub(crate) const REL_RELATIVE: u32 = 8;
pub(crate) const R_RISCV_64: u32 = 2;
pub(crate) const R_RISCV_RELATIVE: u32 = 3;

pub(crate) const AT_PHDR: u8 = 3;
pub(crate) const AT_PHENT: u8 = 4;
pub(crate) const AT_PHNUM: u8 = 5;
pub(crate) const AT_PAGESZ: u8 = 6;
#[allow(unused)]
pub(crate) const AT_BASE: u8 = 7;
#[allow(unused)]
pub(crate) const AT_ENTRY: u8 = 9;
pub(crate) const AT_RANDOM: u8 = 25;

/// PageTable + MemoryArea for a process (task)
pub struct MemorySet {
    page_table: PageTable,
    owned_mem: BTreeMap<usize, MapArea>,
    pub entry: usize,
}

impl MemorySet {
    pub fn page_table_token(&self) -> usize {
        self.page_table.root_paddr().as_usize()
    }

    pub fn new_empty() -> Self {
        Self {
            page_table: PageTable::try_new().expect("Error allocating page table."),
            owned_mem: BTreeMap::new(),
            entry: 0,
        }
    }

    pub fn new_with_kernel_mapped() -> Self {
        let mut page_table = PageTable::try_new().expect("Error allocating page table.");

        for r in memory_regions() {
            debug!(
                "mapping kernel region [0x{:x}, 0x{:x})",
                usize::from(phys_to_virt(r.paddr)),
                usize::from(phys_to_virt(r.paddr)) + r.size,
            );
            page_table
                .map_region(phys_to_virt(r.paddr), r.paddr, r.size, r.flags.into(), true)
                .expect("Error mapping kernel memory");
        }

        Self {
            page_table,
            owned_mem: BTreeMap::new(),
            entry: 0,
        }
    }

    pub fn map_elf(&mut self, elf: &xmas_elf::ElfFile) -> BTreeMap<u8, usize> {
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

        // Some elf will load ELF Header (offset == 0) to vaddr 0. In that case, base_addr will be added to all the LOAD.
        let (base_addr, elf_header_vaddr): (usize, usize) = if let Some(header) = elf
            .program_iter()
            .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Load) && ph.offset() == 0)
        {
            // Loading ELF Header into memory.
            let vaddr = header.virtual_addr() as usize;

            if vaddr == 0 {
                (0x400_0000, 0x400_0000)
            } else {
                (0, vaddr)
            }
        } else {
            (0, 0)
        };
        info!("Base addr for the elf: 0x{:x}", base_addr);

        // Load Elf "LOAD" segments at base_addr.
        elf.program_iter()
            .filter(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Load))
            .for_each(|ph| {
                let mut start_va = ph.virtual_addr() as usize + base_addr;
                let end_va = (ph.virtual_addr() + ph.mem_size()) as usize + base_addr;
                let mut start_offset = ph.offset() as usize;
                let end_offset = (ph.offset() + ph.file_size()) as usize;

                // Virtual address from elf may not be aligned.
                assert_eq!(start_va % PAGE_SIZE_4K, start_offset % PAGE_SIZE_4K);
                let front_pad = start_va % PAGE_SIZE_4K;
                start_va -= front_pad;
                start_offset -= front_pad;

                let mut flags = MappingFlags::USER;
                if ph.flags().is_read() {
                    flags |= MappingFlags::READ;
                }
                if ph.flags().is_write() {
                    flags |= MappingFlags::WRITE;
                }
                if ph.flags().is_execute() {
                    flags |= MappingFlags::EXECUTE;
                }

                debug!(
                    "[new region] elf section [0x{:x}, 0x{:x})",
                    start_va, end_va
                );

                self.new_region(
                    VirtAddr::from(start_va),
                    end_va - start_va,
                    flags,
                    Some(&elf.input[start_offset..end_offset]),
                    None,
                );
            });

        info!("[loader] base addr: 0x{:x}", base_addr);

        // Relocate .rela.dyn sections
        if let Some(rela_dyn) = elf.find_section_by_name(".rela.dyn") {
            let data = match rela_dyn.get_data(&elf) {
                Ok(xmas_elf::sections::SectionData::Rela64(data)) => data,
                _ => panic!("Invalid data in .rela.dyn section"),
            };

            if let Some(dyn_sym_table) = elf.find_section_by_name(".dynsym") {
                let dyn_sym_table = match dyn_sym_table.get_data(&elf) {
                    Ok(xmas_elf::sections::SectionData::DynSymbolTable64(dyn_sym_table)) => {
                        dyn_sym_table
                    }
                    _ => panic!("Invalid data in .dynsym section"),
                };

                info!("Relocating .rela.dyn");
                for entry in data {
                    match entry.get_type() {
                        REL_GOT | REL_PLT | R_RISCV_64 => {
                            let dyn_sym = &dyn_sym_table[entry.get_symbol_table_index() as usize];
                            let sym_val = if dyn_sym.shndx() == 0 {
                                let name = dyn_sym.get_name(&elf).unwrap();
                                panic!(r#"Symbol "{}" not found"#, name);
                            } else {
                                base_addr + dyn_sym.value() as usize
                            };

                            let value = sym_val + entry.get_addend() as usize;
                            let addr = base_addr + entry.get_offset() as usize;

                            info!(
                                "write: {:#x} @ {:#x} type = {}",
                                value,
                                addr,
                                entry.get_type() as usize
                            );

                            unsafe {
                                copy_nonoverlapping(
                                    value.to_ne_bytes().as_ptr(),
                                    addr as *mut u8,
                                    size_of::<usize>() / size_of::<u8>(),
                                );
                            }
                        }
                        REL_RELATIVE | R_RISCV_RELATIVE => {
                            let value = base_addr + entry.get_addend() as usize;
                            let addr = base_addr + entry.get_offset() as usize;

                            info!(
                                "write: {:#x} @ {:#x} type = {}",
                                value,
                                addr,
                                entry.get_type() as usize
                            );

                            unsafe {
                                copy_nonoverlapping(
                                    value.to_ne_bytes().as_ptr(),
                                    addr as *mut u8,
                                    size_of::<usize>() / size_of::<u8>(),
                                );
                            }
                        }
                        other => panic!("Unknown relocation type: {}", other),
                    }
                }
            }
        }

        // Relocate .rela.plt sections
        if let Some(rela_plt) = elf.find_section_by_name(".rela.plt") {
            let data = match rela_plt.get_data(&elf) {
                Ok(xmas_elf::sections::SectionData::Rela64(data)) => data,
                _ => panic!("Invalid data in .rela.plt section"),
            };
            let dyn_sym_table = match elf
                .find_section_by_name(".dynsym")
                .expect("Dynamic Symbol Table not found for .rela.plt section")
                .get_data(&elf)
            {
                Ok(xmas_elf::sections::SectionData::DynSymbolTable64(dyn_sym_table)) => {
                    dyn_sym_table
                }
                _ => panic!("Invalid data in .dynsym section"),
            };

            info!("Relocating .rela.plt");
            for entry in data {
                match entry.get_type() {
                    5 => {
                        let dyn_sym = &dyn_sym_table[entry.get_symbol_table_index() as usize];
                        let sym_val = if dyn_sym.shndx() == 0 {
                            let name = dyn_sym.get_name(&elf).unwrap();
                            panic!(r#"Symbol "{}" not found"#, name);
                        } else {
                            dyn_sym.value() as usize
                        };

                        let value = base_addr + sym_val;
                        let addr = base_addr + entry.get_offset() as usize;

                        info!(
                            "write: {:#x} @ {:#x} type = {}",
                            value,
                            addr,
                            entry.get_type() as usize
                        );

                        unsafe {
                            copy_nonoverlapping(
                                value.to_ne_bytes().as_ptr(),
                                addr as *mut u8,
                                size_of::<usize>(),
                            );
                        }
                    }
                    other => panic!("Unknown relocation type: {}", other),
                }
            }
        }

        info!("Relocating done");
        self.entry = elf.header.pt2.entry_point() as usize + base_addr;

        let mut map = BTreeMap::new();
        map.insert(
            AT_PHDR,
            elf_header_vaddr + elf.header.pt2.ph_offset() as usize,
        );
        map.insert(AT_PHENT, elf.header.pt2.ph_entry_size() as usize);
        map.insert(AT_PHNUM, elf.header.pt2.ph_count() as usize);
        map.insert(AT_RANDOM, 0);
        map.insert(AT_PAGESZ, PAGE_SIZE_4K);
        map
    }

    pub fn page_table_root_ppn(&self) -> PhysAddr {
        self.page_table.root_paddr()
    }

    pub fn max_va(&self) -> VirtAddr {
        self.owned_mem
            .last_key_value()
            .map(|(_, area)| area.end_va())
            .unwrap_or_default()
    }

    /// Allocate contiguous region. If no data, it will create a lazy load region.
    pub fn new_region(
        &mut self,
        vaddr: VirtAddr,
        size: usize,
        flags: MappingFlags,
        data: Option<&[u8]>,
        backend: Option<MemBackend>,
    ) {
        let num_pages = (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K;

        let area = match data {
            Some(data) => MapArea::new_alloc(
                vaddr,
                num_pages,
                flags,
                Some(data),
                backend,
                &mut self.page_table,
            ),
            None => MapArea::new_lazy(vaddr, num_pages, flags, backend, &mut self.page_table),
        };

        debug!(
            "allocating [0x{:x}, 0x{:x}) to [0x{:x}, 0x{:x}) flag: {:?}",
            usize::from(vaddr),
            usize::from(vaddr) + size,
            usize::from(area.vaddr),
            usize::from(area.vaddr) + area.size(),
            flags
        );
        // self.owned_mem.insert(area.vaddr.into(), area);
        assert!(self.owned_mem.insert(area.vaddr.into(), area).is_none());
    }

    /// Make [start, end) unmapped and dealloced. You need to flush TLB after this.
    ///
    /// NOTE: modified map area will have the same PhysAddr.
    fn split_for_area(&mut self, start: VirtAddr, size: usize) {
        let end = start + size;
        assert!(end.is_aligned_4k());

        // Note: Some areas will have to shrink its left part, so its key in BTree (start vaddr) have to change.
        // We get all the overlapped areas out first.
        let overlapped_area: Vec<_> = self
            .owned_mem
            .extract_if(|_, area| area.overlap_with(start, end))
            .collect();

        info!("splitting for [{:?}, {:?})", start, end);

        // Modify areas and insert it back to BTree.
        for (_, mut area) in overlapped_area {
            if area.contained_in(start, end) {
                info!("  drop [{:?}, {:?})", area.vaddr, area.end_va());
                // drop area
                drop(area);
            } else if area.strict_contain(start, end) {
                info!(
                    "  split [{:?}, {:?}) into 2 areas",
                    area.vaddr,
                    area.end_va()
                );
                let new_area = area.remove_mid(start, end, &mut self.page_table);

                assert!(self
                    .owned_mem
                    .insert(new_area.vaddr.into(), new_area)
                    .is_none());
                assert!(self.owned_mem.insert(area.vaddr.into(), area).is_none());
            } else if start <= area.vaddr && area.vaddr < end {
                info!(
                    "  shrink_left [{:?}, {:?}) to [{:?}, {:?})",
                    area.vaddr,
                    area.end_va(),
                    end,
                    area.end_va()
                );
                area.shrink_left(end, &mut self.page_table);

                assert!(self.owned_mem.insert(area.vaddr.into(), area).is_none());
            } else {
                info!(
                    "  shrink_right [{:?}, {:?}) to [{:?}, {:?})",
                    area.vaddr,
                    area.end_va(),
                    area.vaddr,
                    start
                );
                area.shrink_right(start, &mut self.page_table);

                assert!(self.owned_mem.insert(area.vaddr.into(), area).is_none());
            }
        }
    }

    fn find_free_area(&self, hint: VirtAddr, size: usize) -> Option<VirtAddr> {
        let mut last_end = hint.max(axconfig::USER_MEMORY_START.into());
        for area in self.owned_mem.values() {
            if last_end + size <= area.vaddr {
                return Some(last_end);
            }
            last_end = area.end_va();
        }
        None
    }

    /// mmap. You need to flush tlb after this.
    pub fn mmap(
        &mut self,
        start: VirtAddr,
        size: usize,
        flags: MappingFlags,
        fixed: bool,
        backend: Option<MemBackend>,
    ) -> isize {
        // align up to 4k
        let size = (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K * PAGE_SIZE_4K;

        debug!(
            "[mmap] vaddr: [{:?}, {:?}), {:?}, fixed: {}, backend: {}",
            start,
            start + size,
            flags,
            fixed,
            backend.is_some()
        );

        let addr = if fixed {
            self.split_for_area(start, size);

            self.new_region(start, size, flags, None, backend);

            unsafe { riscv::asm::sfence_vma_all() };

            start.as_usize() as isize
        } else {
            info!("find free area");
            let start = self.find_free_area(start, size);

            match start {
                Some(start) => {
                    debug!("found area [{:?}, {:?})", start, start + size);
                    self.new_region(start, size, flags, None, backend);

                    start.as_usize() as isize
                }
                None => -1,
            }
        };

        debug!("[mmap] return addr: 0x{:x}", addr);

        addr
    }

    /// munmap. You need to flush TLB after this.
    pub fn munmap(&mut self, start: VirtAddr, size: usize) {
        // align up to 4k
        let size = (size + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K * PAGE_SIZE_4K;
        info!("[munmap] [{:?}, {:?})", start, (start + size).align_up_4k());

        self.split_for_area(start, size);
    }

    /// msync
    pub fn msync(&mut self, start: VirtAddr, size: usize) {
        let end = start + size;

        for area in self.owned_mem.values_mut() {
            if area.backend.is_none() {
                continue;
            }
            if area.overlap_with(start, end) {
                for page_index in 0..area.pages.len() {
                    let page_vaddr = area.vaddr + page_index * PAGE_SIZE_4K;

                    if page_vaddr >= start && page_vaddr < end {
                        area.sync_page_with_backend(page_index);
                    }
                }
            }
        }
    }

    /// Edit the page table to update flags in given virt address segment. You need to flush TLB
    /// after calling this function.
    ///
    /// NOTE: It's possible that this function will break map areas into two for different mapping
    /// flag settings.
    pub fn mprotect(&mut self, start: VirtAddr, size: usize, flags: MappingFlags) {
        info!(
            "[mprotect] addr: [{:?}, {:?}), flags: {:?}",
            start,
            start + size,
            flags
        );
        let end = start + size;
        assert!(end.is_aligned_4k());

        // NOTE: There will be new areas but all old aree's start address won't change. But we
        // can't iterating through `value_mut()` while `insert()` to BTree at the same time, so we
        // `extract_if()` out the overlapped areas first.
        let overlapped_area: Vec<_> = self
            .owned_mem
            .extract_if(|_, area| area.overlap_with(start, end))
            .collect();

        for (_, mut area) in overlapped_area {
            if area.contained_in(start, end) {
                // update whole area
                area.update_flags(flags, &mut self.page_table);
            } else if area.strict_contain(start, end) {
                // split into 3 areas, update the middle one
                let (mut mid, right) = area.split3(start, end);

                mid.update_flags(flags, &mut self.page_table);

                assert!(self.owned_mem.insert(mid.vaddr.into(), mid).is_none());
                assert!(self.owned_mem.insert(right.vaddr.into(), right).is_none());
            } else if start <= area.vaddr && area.vaddr < end {
                // split into 2 areas, update the left one
                let right = area.split(end);

                area.update_flags(flags, &mut self.page_table);

                assert!(self.owned_mem.insert(right.vaddr.into(), right).is_none());
            } else {
                // split into 2 areas, update the right one
                let mut right = area.split(start);

                right.update_flags(flags, &mut self.page_table);

                assert!(self.owned_mem.insert(right.vaddr.into(), right).is_none());
            }

            assert!(self.owned_mem.insert(area.vaddr.into(), area).is_none());
        }
    }

    /// It will map newly allocated page in the page table. You need to flush TLB after this.
    pub fn handle_page_fault(&mut self, addr: VirtAddr, flags: MappingFlags) {
        match self
            .owned_mem
            .values_mut()
            .find(|area| area.vaddr <= addr && addr < area.end_va())
        {
            Some(area) => area.handle_page_fault(addr, flags, &mut self.page_table),
            None => error!("Page fault address {:?} not found in memory set", addr),
        }
    }

    /// 将用户分配的页面从页表中直接解映射，内核分配的页面依然保留
    pub fn unmap_user_areas(&mut self) {
        for (_, area) in &self.owned_mem {
            self.page_table
                .unmap_region(area.vaddr, area.size())
                .unwrap();
        }
        self.owned_mem.clear();
    }

    /// 判断某一个虚拟地址是否在内存集中。
    /// 若当前虚拟地址在内存集中，且对应的是lazy分配，暂未分配物理页的情况下，
    /// 则为其分配物理页面。
    ///
    /// 若不在内存集中，则返回None。
    ///
    /// 若在内存集中，且已经分配了物理页面，则不做处理。
    pub fn manual_alloc_for_lazy(&mut self, addr: VirtAddr) -> AxResult<()> {
        if let Some((_, area)) = self
            .owned_mem
            .iter_mut()
            .find(|(_, area)| area.vaddr <= addr && addr < area.end_va())
        {
            let entry = self.page_table.get_entry_mut(addr);
            if entry.is_err() {
                // 地址不合法
                return Err(AxError::InvalidInput);
            }

            let entry = entry.unwrap().0;
            if !entry.is_present() {
                // 若未分配物理页面，则手动为其分配一个页面，写入到对应页表中
                area.handle_page_fault(addr, entry.flags(), &mut self.page_table);
            }
            Ok(())
        } else {
            Err(AxError::InvalidInput)
        }
    }

    pub fn query(&self, vaddr: VirtAddr) -> AxResult<(PhysAddr, MappingFlags, PageSize)> {
        if let Ok((paddr, flags, size)) = self.page_table.query(vaddr) {
            Ok((paddr, flags, size))
        } else {
            Err(AxError::InvalidInput)
        }
    }
}

impl Clone for MemorySet {
    fn clone(&self) -> Self {
        let mut page_table = PageTable::try_new().expect("Error allocating page table.");

        for r in memory_regions() {
            debug!(
                "mapping kernel region [0x{:x}, 0x{:x})",
                usize::from(phys_to_virt(r.paddr)),
                usize::from(phys_to_virt(r.paddr)) + r.size,
            );
            page_table
                .map_region(phys_to_virt(r.paddr), r.paddr, r.size, r.flags.into(), true)
                .expect("Error mapping kernel memory");
        }

        let owned_mem = self
            .owned_mem
            .iter()
            .map(|(vaddr, area)| (*vaddr, unsafe { area.clone_alloc(&mut page_table) }))
            .collect();

        Self {
            page_table,
            owned_mem,
            entry: self.entry,
        }
    }
}
