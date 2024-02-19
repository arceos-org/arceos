//! To parse the elf file and map it to the memory space
#![no_std]

extern crate alloc;
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{mem::size_of, ptr::copy_nonoverlapping, str::from_utf8};
use log::info;
use memory_addr::{VirtAddr, PAGE_SIZE_4K};

use axerrno::{AxError, AxResult};

use page_table_entry::MappingFlags;
use xmas_elf::{program::SegmentData, symbol_table::Entry, ElfFile};

mod constant;
use constant::*;
mod user_stack;
use user_stack::init_stack;

/// A trait for mapping ELF file to memory.
pub trait MapELF {
    /// Map ELF file to memory.
    /// It will allocate a physical memory region and map the ELF file to this region.(No Lazy)
    fn map_elf_region(
        &mut self,
        vaddr: VirtAddr,
        size: usize,
        flags: MappingFlags,
        data: Option<&[u8]>,
    );
}

/// A trait to parse the path with the link or mount point
pub trait PathParser {
    /// Read file from fs.
    fn read(&self, _path: &str) -> AxResult<Vec<u8>> {
        // default not support
        Err(AxError::Unsupported)
    }
    /// parse the path by link or mount point and get the real path
    fn real_path(&self, path: &str) -> AxResult<String> {
        // default not support
        Ok(path.to_string())
    }
}

/// Parse the elf file content
///
/// Return the entry point and auxv
fn map_elf_file<T: MapELF>(
    elf: &xmas_elf::ElfFile,
    memory_set: &mut T,
) -> (VirtAddr, BTreeMap<u8, usize>) {
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");

    // Some elf will load ELF Header (offset == 0) to vaddr 0. In that case, base_addr will be added to all the LOAD.
    let (base_addr, elf_header_vaddr): (usize, usize) = if let Some(header) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Load))
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

            memory_set.map_elf_region(
                VirtAddr::from(start_va),
                end_va - start_va,
                flags,
                Some(&elf.input[start_offset..end_offset]),
            );
        });

    // Relocate .rela.dyn sections
    // R_TYPE 与处理器架构有关，相关文档详见
    // x86_64: https://gitlab.com/x86-psABIs/x86-64-ABI/-/jobs/artifacts/master/raw/x86-64-ABI/abi.pdf?job=build
    // riscv64: https://d3s.mff.cuni.cz/files/teaching/nswi200/202324/doc/riscv-abi.pdf
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
                    // #[cfg(target_arch = "x86_64")]
                    R_X86_64_IRELATIVE => {
                        // TODO: 这里的 value 应当是调用 value_function() 得到的内容，但是会导致卡死
                        // let value_function = base_addr + entry.get_addend() as usize;
                        // // 值是在相应的R_X86_64_RELATIVE重定位结果地址处执行的无参数函数返回的程序地址
                        // // 结果地址应当是 base + addend
                        // // let value = unsafe {
                        // //     core::mem::transmute::<_, fn() -> usize>(value_function)
                        // // }();
                        let value = 0 as usize;
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
        if elf.find_section_by_name(".dynsym").is_some() {
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
                    5 | 7 => {
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
    }

    info!("Relocating done");
    let entry = elf.header.pt2.entry_point() as usize + base_addr;

    let mut map = BTreeMap::new();
    map.insert(
        AT_PHDR,
        elf_header_vaddr + elf.header.pt2.ph_offset() as usize,
    );
    map.insert(AT_PHENT, elf.header.pt2.ph_entry_size() as usize);
    map.insert(AT_PHNUM, elf.header.pt2.ph_count() as usize);
    map.insert(AT_RANDOM, 0);
    map.insert(AT_PAGESZ, PAGE_SIZE_4K);
    (entry.into(), map)
}

/// To load the elf file and map it to the memory space
///
/// Return the entry point of the application and the auxv
pub fn parse_elf<T: MapELF, U: PathParser>(
    elf_data: Vec<u8>,
    memory_set: &mut T,
    reader: &U,
) -> AxResult<(VirtAddr, BTreeMap<u8, usize>)> {
    let elf = ElfFile::new(&elf_data).expect("Error parsing app ELF file.");
    if let Some(interp) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
    {
        let interp = match interp.get_data(&elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("Invalid data in Interp Elf Program Header"),
        };

        let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
        // remove trailing '\0'
        let interp_path = interp_path.trim_matches(char::from(0));

        let real_interp_path = reader
            .real_path(interp_path)
            .expect("Error getting real path");
        let interp = reader
            .read(real_interp_path.as_str())
            .expect("Error reading Interpreter from fs");

        return parse_elf(interp, memory_set, reader);
    }
    let (entry, auxv) = map_elf_file(&elf, memory_set);
    Ok((entry, auxv))
}

/// To load the elf file and map it to the memory space
///
/// It will also allocate memory for user stack and heap.
///
/// The heap_loc parameter is a tuple of `(heap_start, heap_size)`.
///
/// The stack_loc parameter is a tuple of `(stack_top, stack_size)`. The stack bottom is `stack_top + stack_size`.
///
/// Return the entry point of the application, user stack bottom and user heap bottom.
pub fn load_elf<T: MapELF, U: PathParser>(
    elf_data: Vec<u8>,
    args: Vec<String>,
    envs: Vec<String>,
    memory_set: &mut T,
    reader: &U,
    heap_loc: (VirtAddr, usize),
    stack_loc: (VirtAddr, usize),
) -> AxResult<(VirtAddr, VirtAddr, VirtAddr)> {
    let elf = ElfFile::new(&elf_data).expect("Error parsing app ELF file.");
    if let Some(interp) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
    {
        let interp = match interp.get_data(&elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("Invalid data in Interp Elf Program Header"),
        };

        let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
        // remove trailing '\0'
        let interp_path = interp_path.trim_matches(char::from(0));

        let mut new_argv = vec![interp_path.to_string()];
        new_argv.extend(args);

        let real_interp_path = reader
            .real_path(interp_path)
            .expect("Error getting real path");
        let interp = reader
            .read(real_interp_path.as_str())
            .expect("Error reading Interpreter from fs");

        return load_elf(
            interp, new_argv, envs, memory_set, reader, heap_loc, stack_loc,
        );
    }

    let (entry, auxv) = map_elf_file(&elf, memory_set);
    // Allocate memory for user stack and hold it in memory_set
    // 栈顶为低地址，栈底为高地址

    let heap_start = heap_loc.0;
    let data = [0 as u8].repeat(heap_loc.1);
    memory_set.map_elf_region(
        heap_start.into(),
        heap_loc.1,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        Some(&data),
    );
    let ustack_top = stack_loc.0;
    let ustack_bottom = ustack_top + stack_loc.1;

    let stack = init_stack(args, envs, auxv, ustack_bottom.into());
    let ustack_bottom = stack.get_sp();
    // 拼接出用户栈初始化数组
    let mut data = [0 as u8].repeat(stack_loc.1 - stack.get_len());
    data.extend(stack.get_data_front_ref());
    memory_set.map_elf_region(
        ustack_top.into(),
        stack_loc.1,
        MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
        Some(&data),
    );
    Ok((entry, ustack_bottom.into(), heap_start.into()))
}
