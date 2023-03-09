use riscv::register::satp;

use axconfig::{PHYS_VIRT_OFFSET, TASK_STACK_SIZE};

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV39: [u64; 512] = [0; 512];

unsafe fn init_mmu() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef;

    let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    extern "Rust" {
        fn rust_main();
    }
    // PC = 0x8020_0000
    // a0 = hartid
    core::arch::asm!("
        mv      s0, a0                  // 0. save hartid
        la      sp, {boot_stack}        // 1. set SP
        li      t0, {boot_stack_size}
        add     sp, sp, t0

        call    {init_mmu}              // 2. setup boot page table and enabel MMU


        la      a1, {platform_init}     // 3. fix up virtual high address
        li      t0, {phys_virt_offset}
        add     a1, a1, t0
        add     sp, sp, t0

        jalr    a1                      // 4. call platform_init()

        la      a1, {rust_main}
        li      t0, {phys_virt_offset}
        add     a1, a1, t0

        mv      a0, s0                  // 5. call rust_main(hartid)
        jalr    a1
        j       .",
        phys_virt_offset = const PHYS_VIRT_OFFSET,
        boot_stack_size = const TASK_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        init_mmu = sym init_mmu,
        platform_init = sym super::platform_init,
        rust_main = sym rust_main,
        options(noreturn),
    )
}
