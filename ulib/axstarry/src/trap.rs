use axhal::{mem::VirtAddr, paging::MappingFlags};

use crate::syscall::syscall;

struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(_irq_num: usize, from_user: bool) {
        #[cfg(feature = "irq")]
        {
            let guard = kernel_guard::NoPreempt::new();
            // trap进来，统计时间信息
            // 只有当trap是来自用户态才进行统计
            if from_user {
                axprocess::time_stat_from_user_to_kernel();
            }
            axhal::irq::dispatch_irq(_irq_num);
            if from_user {
                axprocess::time_stat_from_kernel_to_user();
            }
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
        axprocess::time_stat_from_user_to_kernel();

        handle_page_fault(addr, flags);
        axprocess::time_stat_from_kernel_to_user();
    }

    #[cfg(feature = "signal")]
    fn handle_signal() {
        axprocess::signal::handle_signals();
    }
}
