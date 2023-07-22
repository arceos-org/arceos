use crate::syscall::syscall;
use axhal::arch::{TrapFrame, SIGNAL_RETURN_TRAP};
use axprocess::process::current_process;
use log::error;
/// 宏内核架构下的trap入口
use memory_addr::VirtAddr;
use page_table_entry::MappingFlags;
struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(irq_num: usize) {
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
    fn handle_page_fault(addr: VirtAddr, flags: MappingFlags, tf: &mut TrapFrame) {
        use axprocess::handle_page_fault;
        use axsignal::signal_no::SignalNo;
        use axtask::current;

        use crate::syscall::signal::{syscall_sigreturn, syscall_tkill};
        if addr.as_usize() == SIGNAL_RETURN_TRAP {
            // 说明是信号执行完毕，此时应当执行sig return
            tf.regs.a0 = syscall_sigreturn() as usize;
            return;
        }

        if handle_page_fault(addr, flags).is_err() {
            // 如果处理失败，则发出sigsegv信号
            let curr = current().id().as_u64() as isize;
            axlog::error!("kill task: {}", curr);
            syscall_tkill(curr, SignalNo::SIGSEGV as isize);
        }
    }

    fn handle_access_fault(addr: VirtAddr, flags: MappingFlags) {
        let process = current_process();
        let inner = process.inner.lock();
        let ans = inner.memory_set.lock().query(addr);
        if ans.is_err() {
            panic!("addr not exist: addr: {:X?}, flags: {:?}", addr, flags);
        }
        let (phy_addr, flags, _) = ans.unwrap();

        panic!("addr: {:X}, flags: {:?}", phy_addr, flags);
    }

    #[cfg(feature = "signal")]
    fn handle_signal() {
        axprocess::handle_signals();
    }

    fn exit() {
        use crate::syscall::signal::syscall_tkill;
        use axsignal::signal_no::SignalNo;
        use axtask::current;
        let curr = current().id().as_u64() as isize;
        axlog::error!("kill task: {}", curr);
        syscall_tkill(curr, SignalNo::SIGSEGV as isize);
    }
}
