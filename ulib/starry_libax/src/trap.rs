use crate::syscall::syscall;

/// 宏内核架构下的trap入口
use memory_addr::VirtAddr;
use page_table_entry::MappingFlags;
struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(_irq_num: usize) {
        #[cfg(feature = "irq")]
        {
            let guard = kernel_guard::NoPreempt::new();
            // trap进来，统计时间信息
            axprocess::time_stat_from_user_to_kernel();
            axhal::irq::dispatch_irq(irq_num);
            axprocess::time_stat_from_kernel_to_user();
            drop(guard); // rescheduling may occur when preemption is re-enabled.
        }
    }
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
        axprocess::time_stat_from_user_to_kernel();
        let ans = syscall(syscall_id, args);
        axprocess::time_stat_from_kernel_to_user();
        ans
    }

    #[cfg(feature = "paging")]
    fn handle_page_fault(addr: VirtAddr, flags: MappingFlags) {
        use axprocess::handle_page_fault;
        handle_page_fault(addr, flags);
    }

    #[cfg(feature = "signal")]
    fn handle_signal() {
        axprocess::handle_signals();
    }
}
