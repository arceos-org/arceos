struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(irq_num: usize) {
        let guard = kernel_guard::NoPreempt::new();
        axhal::irq::dispatch_irq(irq_num);
        drop(guard); // rescheduling may occur when preemption is re-enabled.
    }
    #[cfg(feature = "user")]
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
        axtask::syscall::syscall(syscall_id, args)
    }
}
