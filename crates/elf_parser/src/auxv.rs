//! Some constant in the elf file
extern crate alloc;
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use log::info;
use memory_addr::{VirtAddr, PAGE_SIZE_4K};

use crate::user_stack::init_stack;
const AT_PHDR: u8 = 3;
const AT_PHENT: u8 = 4;
const AT_PHNUM: u8 = 5;
const AT_PAGESZ: u8 = 6;
#[allow(unused)]
const AT_BASE: u8 = 7;
#[allow(unused)]
const AT_ENTRY: u8 = 9;
const AT_RANDOM: u8 = 25;

/// To parse the elf file and get the auxv vectors
///
/// # Arguments
///
/// * `elf` - The elf file
/// * `elf_base_addr` - The base address of the elf file if the file will be loaded to the memory
pub fn get_auxv_vector(
    elf: &xmas_elf::ElfFile,
    elf_base_addr: Option<usize>,
) -> BTreeMap<u8, usize> {
    // Some elf will load ELF Header (offset == 0) to vaddr 0. In that case, base_addr will be added to all the LOAD.
    let elf_header_vaddr: usize = if let Some(header) = elf
        .program_iter()
        .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Load))
    {
        // Loading ELF Header into memory.
        let vaddr = header.virtual_addr() as usize;

        if vaddr == 0 {
            if elf_base_addr.is_some() {
                elf_base_addr.unwrap()
            } else {
                panic!("ELF Header is loaded to vaddr 0, but no base_addr is provided");
            }
        } else {
            vaddr
        }
    } else {
        0
    };
    info!("ELF header addr: 0x{:x}", elf_header_vaddr);
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
/// To get the app stack and the information on the stack from the ELF file
///
/// # Arguments
///
/// * `args` - The arguments of the app
/// * `envs` - The environment variables of the app
/// * `auxv` - The auxv vector of the app
/// * `stack_top` - The top address of the stack
/// * `stack_size` - The size of the stack.
///
/// # Return
///
/// `(stack_content, real_stack_bottom)`
///
/// * `stack_content`: the stack data from the low address to the high address, which will be used to map in the memory
///
/// * `real_stack_bottom`: The initial stack bottom is `stack_top + stack_size`.After push arguments into the stack, it will return the real stack bottom
///
/// The return data will be divided into two parts.
/// * The first part is the free stack content, which is all 0.
/// * The second part is the content carried by the user stack when it is initialized, such as args, auxv, etc.
///
/// The detailed format is described in https://articles.manugarg.com/aboutelfauxiliaryvectors.html
pub fn get_app_stack_region(
    args: Vec<String>,
    envs: Vec<String>,
    auxv: BTreeMap<u8, usize>,
    stack_top: VirtAddr,
    stack_size: usize,
) -> (Vec<u8>, usize) {
    let ustack_top = stack_top;
    let ustack_bottom = ustack_top + stack_size;
    // The stack variable is actually the information carried by the stack
    let stack = init_stack(args, envs, auxv, ustack_bottom.into());
    let ustack_bottom = stack.get_sp();
    let mut data = [0 as u8].repeat(stack_size - stack.get_len());
    data.extend(stack.get_data_front_ref());
    (data, ustack_bottom)
}
