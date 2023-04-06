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
    extern "Rust" {
        fn __user_start();
    }
    use axhal::arch::TrapFrame;
    const STACK_SIZE: usize = 4096;
    static KERNEL_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    static USER_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    
    let trap_frame: TrapFrame = TrapFrame::new(__user_start as usize, USER_STACK.as_ptr() as usize + STACK_SIZE);
    info!("Into User state.");
    trap_frame.enter_uspace(KERNEL_STACK.as_ptr() as usize + STACK_SIZE)
}
