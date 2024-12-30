use crate::mem::{PhysAddr, phys_to_virt, virt_to_phys};

static mut SECONDARY_STACK_TOP: usize = 0;

unsafe extern "C" {
    fn _start_secondary();
}

#[naked]
#[unsafe(link_section = ".text.boot")]
unsafe extern "C" fn modify_stack_and_start() {
    core::arch::naked_asm!("
        ldr     x21, ={secondary_boot_stack}    // the secondary CPU hasn't set the TTBR1
        mov     x8, {phys_virt_offset}          // minus the offset to get the phys addr of the boot stack
        sub     x21, x21, x8
        ldr     x21, [x21]
        mov     x0, x21                         // x0 will be set to SP in the beginning of _start_secondary
        b       _start_secondary",
        secondary_boot_stack = sym SECONDARY_STACK_TOP,
        phys_virt_offset = const axconfig::plat::PHYS_VIRT_OFFSET,
    );
}

pub static CPU_SPIN_TABLE: [PhysAddr; 4] = [pa!(0xd8), pa!(0xe0), pa!(0xe8), pa!(0xf0)];

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(cpu_id: usize, stack_top: PhysAddr) {
    let entry_paddr = virt_to_phys(va!(modify_stack_and_start as usize)).as_usize();
    unsafe {
        // set the boot code address of the given secondary CPU
        let spintable_vaddr = phys_to_virt(CPU_SPIN_TABLE[cpu_id]);
        let release_ptr = spintable_vaddr.as_mut_ptr() as *mut usize;
        release_ptr.write_volatile(entry_paddr);
        crate::arch::flush_dcache_line(spintable_vaddr);

        // set the boot stack of the given secondary CPU
        SECONDARY_STACK_TOP = stack_top.as_usize();
        crate::arch::flush_dcache_line(va!(core::ptr::addr_of!(SECONDARY_STACK_TOP) as usize));
    }
    aarch64_cpu::asm::sev();
}
