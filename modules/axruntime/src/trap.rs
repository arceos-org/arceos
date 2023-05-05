struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(irq_num: usize) {
        let guard = kernel_guard::NoPreempt::new();
        // trap进来，统计时间信息
        axprocess::time_stat_from_user_to_kernel();
        axhal::irq::dispatch_irq(irq_num);
        axprocess::time_stat_from_kernel_to_user();
        drop(guard); // rescheduling may occur when preemption is re-enabled.
    }
    #[cfg(feature = "user")]
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
        axprocess::time_stat_from_user_to_kernel();
        let ans = axsyscall::syscall(syscall_id, args);
        axprocess::time_stat_from_kernel_to_user();
        ans
    }
}
