//! Some constant in the elf file

pub(crate) const REL_GOT: u32 = 6;
pub(crate) const REL_PLT: u32 = 7;
pub(crate) const REL_RELATIVE: u32 = 8;
pub(crate) const R_RISCV_64: u32 = 2;
pub(crate) const R_RISCV_RELATIVE: u32 = 3;

// #[cfg(target_arch = "x86_64")]
pub(crate) const R_X86_64_IRELATIVE: u32 = 37;

pub(crate) const AT_PHDR: u8 = 3;
pub(crate) const AT_PHENT: u8 = 4;
pub(crate) const AT_PHNUM: u8 = 5;
pub(crate) const AT_PAGESZ: u8 = 6;
#[allow(unused)]
pub(crate) const AT_BASE: u8 = 7;
#[allow(unused)]
pub(crate) const AT_ENTRY: u8 = 9;
pub(crate) const AT_RANDOM: u8 = 25;
