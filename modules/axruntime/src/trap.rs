struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(irq_num: usize) {
        let guard = kernel_guard::NoPreempt::new();
        axhal::irq::dispatch_irq(irq_num);
        drop(guard); // rescheduling may occur when preemption is re-enabled.
    }

    #[cfg(feature = "user")]
    fn handle_syscall(syscall_num: usize, params: [usize; 6]) -> isize {
        crate::syscall::syscall_handler(syscall_num, params)
    }
}

#[cfg(feature = "user")]
pub fn user_space_entry() -> ! {
    if cfg!(feature = "user-paging") {
        info!("Into User State");
        axhal::arch::enter_uspace(
            0xffff_ffff_fff0_0000,
            axmem::get_satp()
        )
    } else {
        extern "Rust" {
            fn __user_start();
        }
        use axhal::arch::TrapFrame;
        use crate::{USTACK_SIZE, USTACK_START, USER_START};
        const STACK_SIZE: usize = 4096;
        // In detailed page table, we distinguish .data(.bss) and .rodata
        // for whether being able to write.
        static mut KERNEL_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];    
        
        let trap_frame: TrapFrame = TrapFrame::new(USER_START, USTACK_START + USTACK_SIZE);
        info!("Into User state.");
        trap_frame.enter_uspace(unsafe { KERNEL_STACK.as_ptr() } as usize + STACK_SIZE)
    }
}
