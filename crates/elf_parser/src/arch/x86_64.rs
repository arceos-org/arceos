//! Relocate .rela.dyn sections
//! R_TYPE 与处理器架构有关，相关文档详见
//! x86_64: https://gitlab.com/x86-psABIs/x86-64-ABI/-/jobs/artifacts/master/raw/x86-64-ABI/abi.pdf?job=build
use core::mem::size_of;

use super::RelocatePair;
use alloc::vec::Vec;
use log::info;
use memory_addr::VirtAddr;
use xmas_elf::symbol_table::Entry;
extern crate alloc;

const R_X86_64_64: u32 = 1;
const R_X86_64_PC32: u32 = 2;
const R_X86_64_GLOB_DAT: u32 = 6;
const R_X86_64_JUMP_SLOT: u32 = 7;
const R_X86_64_RELATIVE: u32 = 8;

const R_X86_64_IRELATIVE: u32 = 37;

/// To parse the elf file and get the relocate pairs
///
/// # Arguments
///
/// * `elf` - The elf file
/// * `elf_base_addr` - The base address of the elf file if the file will be loaded to the memory
pub fn get_relocate_pairs(
    elf: &xmas_elf::ElfFile,
    elf_base_addr: Option<usize>,
) -> Vec<RelocatePair> {
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
    let mut pairs = Vec::new();
    // Some elf will load ELF Header (offset == 0) to vaddr 0. In that case, base_addr will be added to all the LOAD.
    let base_addr: usize = if let Some(header) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Load))
    {
        // Loading ELF Header into memory.
        let vaddr = header.virtual_addr() as usize;

        if vaddr == 0 {
            if let Some(addr) = elf_base_addr {
                addr
            } else {
                panic!("ELF Header is loaded to vaddr 0, but no base_addr is provided");
            }
        } else {
            0
        }
    } else {
        0
    };
    info!("Base addr for the elf: 0x{:x}", base_addr);
    if let Some(rela_dyn) = elf.find_section_by_name(".rela.dyn") {
        let data = match rela_dyn.get_data(elf) {
            Ok(xmas_elf::sections::SectionData::Rela64(data)) => data,
            _ => panic!("Invalid data in .rela.dyn section"),
        };

        if let Some(dyn_sym_table) = elf.find_section_by_name(".dynsym") {
            let dyn_sym_table = match dyn_sym_table.get_data(elf) {
                Ok(xmas_elf::sections::SectionData::DynSymbolTable64(dyn_sym_table)) => {
                    dyn_sym_table
                }
                _ => panic!("Invalid data in .dynsym section"),
            };
            info!("Relocating .rela.dyn");
            for entry in data {
                let dyn_sym = &dyn_sym_table[entry.get_symbol_table_index() as usize];
                let destination = base_addr + entry.get_offset() as usize;
                let symbol_value = if dyn_sym.shndx() != 0 {
                    dyn_sym.value() as usize
                } else {
                    let name = dyn_sym.get_name(elf).unwrap();
                    panic!(r#"Symbol "{}" not found"#, name);
                }; // Represents the value of the symbol whose index resides in the relocation entry.
                let addend = entry.get_addend() as usize; // Represents the addend used to compute the value of the relocatable field.
                match entry.get_type() {
                    R_X86_64_64 => {
                        if dyn_sym.shndx() == 0 {
                            let name = dyn_sym.get_name(elf).unwrap();
                            panic!(r#"Symbol "{}" not found"#, name);
                        };
                        pairs.push(RelocatePair {
                            src: VirtAddr::from(symbol_value),
                            dst: VirtAddr::from(destination),
                            count: size_of::<usize>() / size_of::<u8>(),
                        })
                    }
                    R_X86_64_PC32 => pairs.push(RelocatePair {
                        src: VirtAddr::from(symbol_value + addend - destination),
                        dst: VirtAddr::from(destination),
                        count: 4,
                    }),
                    R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
                        if dyn_sym.shndx() == 0 {
                            let name = dyn_sym.get_name(elf).unwrap();
                            panic!(r#"Symbol "{}" not found"#, name);
                        };
                        pairs.push(RelocatePair {
                            src: VirtAddr::from(symbol_value),
                            dst: VirtAddr::from(destination),
                            count: size_of::<usize>() / size_of::<u8>(),
                        })
                    }
                    R_X86_64_RELATIVE => pairs.push(RelocatePair {
                        src: VirtAddr::from(base_addr + addend),
                        dst: VirtAddr::from(destination),
                        count: size_of::<usize>() / size_of::<u8>(),
                    }),

                    R_X86_64_IRELATIVE => {
                        // TODO: 这里的 value 应当是调用 value_function() 得到的内容，但是会导致卡死
                        // let value_function = base_addr + entry.get_addend() as usize;
                        // // 值是在相应的R_X86_64_RELATIVE重定位结果地址处执行的无参数函数返回的程序地址
                        // // let value = unsafe {
                        // //     core::mem::transmute::<_, fn() -> usize>(value_function)
                        // // }();
                        let value = 0;
                        pairs.push(RelocatePair {
                            src: VirtAddr::from(value),
                            dst: VirtAddr::from(destination),
                            count: size_of::<usize>() / size_of::<u8>(),
                        });
                    }
                    other => panic!("Unknown relocation type: {}", other),
                }
            }
        }
    }

    // Relocate .rela.plt sections
    if let Some(rela_plt) = elf.find_section_by_name(".rela.plt") {
        let data = match rela_plt.get_data(elf) {
            Ok(xmas_elf::sections::SectionData::Rela64(data)) => data,
            _ => panic!("Invalid data in .rela.plt section"),
        };
        if elf.find_section_by_name(".dynsym").is_some() {
            let dyn_sym_table = match elf
                .find_section_by_name(".dynsym")
                .expect("Dynamic Symbol Table not found for .rela.plt section")
                .get_data(elf)
            {
                Ok(xmas_elf::sections::SectionData::DynSymbolTable64(dyn_sym_table)) => {
                    dyn_sym_table
                }
                _ => panic!("Invalid data in .dynsym section"),
            };

            info!("Relocating .rela.plt");
            for entry in data {
                match entry.get_type() {
                    R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
                        let dyn_sym = &dyn_sym_table[entry.get_symbol_table_index() as usize];
                        let destination = base_addr + entry.get_offset() as usize;
                        let symbol_value = if dyn_sym.shndx() != 0 {
                            dyn_sym.value() as usize
                        } else {
                            let name = dyn_sym.get_name(elf).unwrap();
                            panic!(r#"Symbol "{}" not found"#, name);
                        }; // Represents the value of the symbol whose index resides in the relocation entry.
                        pairs.push(RelocatePair {
                            src: VirtAddr::from(symbol_value),
                            dst: VirtAddr::from(destination),
                            count: size_of::<usize>() / size_of::<u8>(),
                        })
                    }
                    other => panic!("Unknown relocation type: {}", other),
                }
            }
        }
    }

    info!("Relocating done");
    pairs
}
