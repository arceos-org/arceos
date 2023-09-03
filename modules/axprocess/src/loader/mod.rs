/// 用于进行文件加载
extern crate alloc;
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
// 堆和栈的基地址
pub const USER_HEAP_OFFSET: usize = 0x3FA0_0000;
pub const USER_STACK_OFFSET: usize = 0x3FE0_0000;
pub const MAX_HEAP_SIZE: usize = 0x40_0000;
pub const USER_STACK_SIZE: usize = 0x20_0000;
use axerrno::AxResult;
mod user_stack;
use axhal::{mem::VirtAddr, paging::MappingFlags};
use axlog::info;
use axmem::MemorySet;
use core::str::from_utf8;
use xmas_elf::{program::SegmentData, ElfFile};

use crate::{
    link::{real_path, FilePath},
    loader::user_stack::init_stack,
};

/// A elf file wrapper.
pub struct Loader<'a> {
    elf: ElfFile<'a>,
}

impl<'a> Loader<'a> {
    /// Create a new Loader from data: &[u8].
    ///
    /// # Panics
    ///
    /// Panics if data is not valid elf.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            elf: ElfFile::new(data).expect("Error parsing app ELF file."),
        }
    }

    /// Create a Task from Loader, only used for init process. Other processes are spawned by
    /// clone (fork) + execve.
    /// This function will allocate kernel stack and put `TrapFrame` (including `argv`) into place.
    /// 返回应用程序入口，用户栈底，用户堆底
    pub fn load(
        self,
        args: Vec<String>,
        envs: Vec<String>,
        mut memory_set: &mut MemorySet,
    ) -> AxResult<(VirtAddr, VirtAddr, VirtAddr)> {
        info!("args: {:?}", args);
        if let Some(interp) = self
            .elf
            .program_iter()
            .find(|ph| ph.get_type() == Ok(xmas_elf::program::Type::Interp))
        {
            let interp = match interp.get_data(&self.elf) {
                Ok(SegmentData::Undefined(data)) => data,
                _ => panic!("Invalid data in Interp Elf Program Header"),
            };

            let interp_path = from_utf8(interp).expect("Interpreter path isn't valid UTF-8");
            // remove trailing '\0'
            let interp_path = interp_path.trim_matches(char::from(0));
            info!("Interpreter path: {}", interp_path);

            let mut new_argv = vec![interp_path.to_string()];
            new_argv.extend(args);
            info!("Interpreter args: {:?}", new_argv);

            #[cfg(not(feature = "fs"))]
            {
                panic!("ELF Interpreter is not supported without fs feature");
            }
            let interp_path = FilePath::new(interp_path)?;
            let real_interp_path = real_path(&(interp_path.path().to_string()));
            let interp = axfs::api::read(real_interp_path.as_str())
                .expect("Error reading Interpreter from fs");
            let loader = Loader::new(&interp);
            return loader.load(new_argv, envs, &mut memory_set);
        }

        let auxv = memory_set.map_elf(&self.elf);
        // Allocate memory for user stack and hold it in memory_set
        // 栈顶为低地址，栈底为高地址

        let heap_start = VirtAddr::from(USER_HEAP_OFFSET);
        let data = [0 as u8].repeat(MAX_HEAP_SIZE);
        memory_set.new_region(
            heap_start,
            MAX_HEAP_SIZE,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
            Some(&data),
            None,
        );
        info!(
            "[new region] user heap: [{:?}, {:?})",
            heap_start,
            heap_start + MAX_HEAP_SIZE
        );
        let ustack_top = VirtAddr::from(USER_STACK_OFFSET);
        let ustack_bottom = ustack_top + USER_STACK_SIZE;
        info!(
            "[new region] user stack: [{:?}, {:?})",
            ustack_top,
            ustack_bottom.align_up_4k()
        );

        let stack = init_stack(args, envs, auxv, ustack_bottom.into());
        let ustack_bottom: VirtAddr = stack.get_sp().into();
        // 拼接出用户栈初始化数组
        let mut data = [0 as u8].repeat(USER_STACK_SIZE - stack.get_len());
        data.extend(stack.get_data_front_ref());
        memory_set.new_region(
            ustack_top,
            USER_STACK_SIZE,
            MappingFlags::USER | MappingFlags::READ | MappingFlags::WRITE,
            Some(&data),
            None,
        );
        Ok((
            memory_set.entry.into(),
            ustack_bottom.into(),
            heap_start.into(),
        ))
    }
}
