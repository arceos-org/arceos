use axhal::mem::{memory_regions, phys_to_virt};
use axhal::paging::{PageTable, ENTRY_COUNT};
use lazy_init::LazyInit;
use memory_addr::PAGE_SIZE_4K;

/// 负责分页机制的实现

pub static KERNEL_PAGE_TABLE: LazyInit<PageTable> = LazyInit::new();

pub fn remap_kernel_memory() -> Result<(), axhal::paging::PagingError> {
    if axhal::cpu::this_cpu_is_bsp() {
        // 映射内核代码
        let mut kernel_page_table = PageTable::try_new()?;
        for r in memory_regions() {
            kernel_page_table.map_region(
                phys_to_virt(r.paddr),
                r.paddr,
                r.size,
                r.flags.into(),
                true,
            )?;
        }
        KERNEL_PAGE_TABLE.init_by(kernel_page_table);
    }
    unsafe { axhal::arch::write_page_table_root(KERNEL_PAGE_TABLE.root_paddr()) };
    Ok(())
}

/// 复制内核页表到用户页表
pub fn copy_from_kernel_memory() -> PageTable {
    let page_table = PageTable::try_new().unwrap();
    let idx_len = PAGE_SIZE_4K / ENTRY_COUNT;
    for idx in 256usize..512 {
        // 由内核初始的虚拟地址决定复制多少大页
        // 一共253GB大页，真有你的
        let kernel_pte_address: *const usize = (phys_to_virt(KERNEL_PAGE_TABLE.root_paddr())
            .as_usize()
            + idx_len * idx) as *const usize;
        let kernel_pte = unsafe { core::slice::from_raw_parts(kernel_pte_address, idx_len) };
        let user_pte_address =
            (phys_to_virt(page_table.root_paddr()).as_usize() + idx_len * idx) as *mut usize;
        let user_pte = unsafe { core::slice::from_raw_parts_mut(user_pte_address, idx_len) };
        user_pte.copy_from_slice(&kernel_pte);
    }
    page_table
}
