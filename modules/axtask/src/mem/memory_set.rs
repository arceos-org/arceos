use crate::{
    mem::{areas::MapArea, paging::copy_from_kernel_memory},
    process::USER_STACK_SIZE,
};
use alloc::{string::String, vec::Vec};
use axalloc::GlobalPage;
use axhal::{
    mem::{virt_to_phys, VirtAddr, PAGE_SIZE_4K},
    paging::{MappingFlags, PageSize, PageTable, PagingResult},
};
use memory_addr::PhysAddr;
/// 地址空间实现
pub struct MemorySet {
    pub page_table: PageTable,
    pub areas: Vec<MapArea>,
}

pub fn get_app_data(_name: &str) -> &'static [u8] {
    extern "C" {
        fn app0_start();
        fn app0_end();
        fn app1_start();
        fn app1_end();
    }
    unsafe {
        if _name == "helloworld" {
            core::slice::from_raw_parts(
                app0_start as usize as *const u8,
                app0_end as usize - app0_start as usize,
            )
        } else {
            core::slice::from_raw_parts(
                app1_start as usize as *const u8,
                app1_end as usize - app1_start as usize,
            )
        }
    }
}

impl MemorySet {
    pub fn new_from_kernel() -> Self {
        Self {
            page_table: copy_from_kernel_memory(),
            areas: Vec::new(),
        }
    }
    pub fn new_empty() -> Self {
        Self {
            page_table: PageTable::try_new().unwrap(),
            areas: Vec::new(),
        }
    }
    /// 从已有任务复制完整的地址空间过来
    /// 1. 对内核的地址段，所有虚拟地址与物理地址的映射相同
    /// 2. 对用户的地址段，所有虚拟地址和其中的数据相同，但对应的物理地址与 self 中的不同
    pub fn new_from_task(others: &Self) -> Self {
        let mut new_memory_set = Self::new_from_kernel();
        for area in others.areas.iter() {
            let data = area.pages.as_slice();
            // 为新的地址空间复制原先地址空间的内容
            new_memory_set.map_region_4k(area.start_va, area.pages.size(), area.flags, Some(data));
        }
        new_memory_set
    }
    /// 获取页表token
    pub fn page_table_token(&self) -> usize {
        self.page_table.root_paddr().as_usize()
    }
    pub fn from_elf(memory_set: &mut MemorySet, elf_data: &[u8]) -> (usize, usize) {
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_va: usize = 0;
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let mem_size = ph.mem_size() as usize;
                let end_va: usize = (ph.virtual_addr() + ph.mem_size()) as usize;
                axlog::info!("start: {:X}, end: {:X}", start_va.as_usize(), end_va);
                if end_va > max_end_va {
                    max_end_va = end_va;
                }
                let mut map_perm = MappingFlags::USER;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MappingFlags::READ;
                }
                if ph_flags.is_write() {
                    map_perm |= MappingFlags::WRITE;
                }
                if ph_flags.is_execute() {
                    map_perm |= MappingFlags::EXECUTE;
                }
                memory_set.map_region_4k(
                    start_va,
                    mem_size,
                    map_perm,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        // 向上取整4K
        let mut user_stack_bottom = (max_end_va + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K * PAGE_SIZE_4K;
        // guard page
        user_stack_bottom += PAGE_SIZE_4K;

        let map_perm = MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER;
        memory_set.map_region_4k(user_stack_bottom.into(), USER_STACK_SIZE, map_perm, None);
        (elf_header.pt2.entry_point() as usize, user_stack_bottom)
    }
    /// 将用户分配的页面从页表中直接解映射，内核分配的页面依然保留
    pub fn unmap_user_areas(&mut self) {
        for area in &self.areas {
            self.page_table
                .unmap_region(area.start_va, area.pages.size())
                .unwrap();
        }
    }
    pub fn map_region_4k(
        &mut self,
        start_va: VirtAddr,
        size: usize,
        map_perm: MappingFlags,
        data: Option<&[u8]>,
    ) {
        // 为每一个新的区域都要进行页面的分配
        // 每一个区域直接连续分配页面
        let num_pages: usize = (size as usize + PAGE_SIZE_4K - 1) / PAGE_SIZE_4K;
        let mut pages = GlobalPage::alloc_contiguous(num_pages, PAGE_SIZE_4K)
            .expect("Failed to get physical pages!");
        pages.zero();
        if let Some(x) = data {
            // 由于是连续的页面，所以可以直接拷贝数据进去
            pages.as_slice_mut()[..x.len()].copy_from_slice(x);
        }
        // 进行页表的映射
        self.page_table
            .map_region(
                start_va.align_down_4k(),
                pages.start_paddr(virt_to_phys),
                pages.size(),
                map_perm,
                false,
            )
            .expect("Error when mapping!");
        self.areas.push(MapArea::new(pages, map_perm, start_va.align_down_4k()));
    }
    pub fn translate(
        &self,
        start_va: VirtAddr,
    ) -> PagingResult<(PhysAddr, MappingFlags, PageSize)> {
        self.page_table.query(start_va)
    }
    /// 在当前地址空间下，将vaddr转化为真实的物理地址
    #[allow(unused)]
    pub fn translate_va(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        match self.page_table.query(vaddr) {
            Ok((paddr, _, _)) => Some(paddr),
            Err(x) => None,
        }
    }
    // pub fn translate_ref<T>(&self, ptr: *const T) -> PagingResult<&'static T> {
    //     let start_va: VirtAddr = (ptr as usize).into();
    //     if start_va.align_down_4k().as_usize() == 0x80202000 {
    //         let x = 0x8020200D;
    //         axlog::info!(
    //             "{:X}",
    //             self.page_table.query(x.into()).unwrap().0.as_usize()
    //         );
    //     }
    //     axlog::info!("now virt addr: {:X}", start_va.as_usize());
    //     match self.page_table.query(start_va) {
    //         Ok((paddr, _, _)) => {
    //             let x = unsafe { (paddr.as_usize() as *const T).as_ref().unwrap() };
    //             // axlog::info!("{:X}", )
    //             return Ok(unsafe { (paddr.as_usize() as *const T).as_ref().unwrap() });
    //         }
    //         Err(x) => {
    //             axlog::info!("Error in va: {}", start_va.as_usize());
    //             return Err(x);
    //         }
    //     }
    // }

    pub fn translate_refmut<T>(&self, ptr: *mut T) -> PagingResult<&'static mut T> {
        let start_va: VirtAddr = (ptr as usize).into();
        match self.page_table.query(start_va) {
            Ok((paddr, _, _)) => {
                return Ok(unsafe { (paddr.as_usize() as *mut T).as_mut().unwrap() })
            }
            Err(x) => return Err(x),
        }
    }
    pub fn translate_str(&self, ptr: *const u8) -> String {
        let mut string = String::new();
        let mut va: usize = ptr as usize;
        loop {
            let ch: u8 = unsafe { *(va as *const u8) };
            if ch == 0 {
                break;
            }
            string.push(ch as char);
            va += 1;
        }
        string
    }
}
