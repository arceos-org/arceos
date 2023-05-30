use axconfig::{SMP, TASK_STACK_SIZE};
use axhal::mem::{virt_to_phys, VirtAddr};
use core::sync::atomic::{AtomicUsize, Ordering};

#[link_section = ".bss.stack"]
static mut SECONDARY_BOOT_STACK: [[u8; TASK_STACK_SIZE]; SMP - 1] = [[0; TASK_STACK_SIZE]; SMP - 1];

static ENTERED_CPUS: AtomicUsize = AtomicUsize::new(1);

#[cfg(all(feature = "smp", not(feature = "platform-raspi4-aarch64")))]
pub fn start_secondary_cpus(primary_cpu_id: usize) {
    let mut logic_cpu_id = 0;
    for i in 0..SMP {
        if i != primary_cpu_id {
            let stack_top = virt_to_phys(VirtAddr::from(unsafe {
                SECONDARY_BOOT_STACK[logic_cpu_id].as_ptr_range().end as usize
            }));

            debug!("starting CPU {}...", i);
            axhal::mp::start_secondary_cpu(i, stack_top);
            logic_cpu_id += 1;

            while ENTERED_CPUS.load(Ordering::Acquire) <= logic_cpu_id {
                core::hint::spin_loop();
            }
        }
    }
}

#[cfg(all(feature = "smp", feature = "platform-raspi4-aarch64"))]
use aarch64_cpu::asm;
#[cfg(all(feature = "smp", feature = "platform-raspi4-aarch64"))]
use core::ptr::write_volatile;

extern "C" {
    fn _start_secondary();
}

#[cfg(all(feature = "smp", feature = "platform-raspi4-aarch64"))]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn modify_stack_and_start() {
    core::arch::asm!("
        mrs     x19, mpidr_el1
        and     x19, x19, #0xffffff             // get current CPU id
        lsl     x20, x19, #18
        ldr     x21, ={secondary_boot_stack}    // core0 store the stack start adress at this address
        mov     x8, {phys_virt_offset} 
        sub     x21, x21, x8
        add     sp, x21, x20
        b       {start_secondary}",
        secondary_boot_stack = sym SECONDARY_BOOT_STACK,
        start_secondary = sym _start_secondary,
        phys_virt_offset = const axconfig::PHYS_VIRT_OFFSET,
    );
}

#[no_mangle]
pub static CPU_SPIN_TABLE: [usize; 4] = [0xd8, 0xe0, 0xe8, 0xf0];

#[cfg(all(feature = "smp", feature = "platform-raspi4-aarch64"))]
pub fn start_secondary_cpus(primary_cpu_id: usize) {
    let entry_address = virt_to_phys(VirtAddr::from(modify_stack_and_start as usize)).as_usize();
    for (i, _) in CPU_SPIN_TABLE.iter().enumerate().take(SMP) {
        if i != primary_cpu_id {
            let release_addr = CPU_SPIN_TABLE[i] as *mut usize;
            unsafe {
                write_volatile(release_addr, entry_address);
            }
            axhal::arch::flush_dcache(release_addr as usize);
        }
    }
    asm::sev();
}

/// The main entry point of the ArceOS runtime for secondary CPUs.
///
/// It is called from the bootstrapping code in [axhal].
#[no_mangle]
pub extern "C" fn rust_main_secondary(cpu_id: usize) -> ! {
    ENTERED_CPUS.fetch_add(1, Ordering::Relaxed);
    info!("Secondary CPU {} started.", cpu_id);

    #[cfg(feature = "paging")]
    super::remap_kernel_memory().unwrap();

    axhal::platform_init_secondary();

    #[cfg(feature = "multitask")]
    axtask::init_scheduler_secondary();

    info!("Secondary CPU {} init OK.", cpu_id);
    super::INITED_CPUS.fetch_add(1, Ordering::Relaxed);

    while !super::is_init_ok() {
        core::hint::spin_loop();
    }

    axhal::arch::enable_irqs();

    #[cfg(feature = "multitask")]
    axtask::run_idle();
    #[cfg(not(feature = "multitask"))]
    loop {
        axhal::arch::wait_for_irqs();
    }
}
