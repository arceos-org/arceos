/// 仅用作非宏内核下的trap入口

struct TrapHandlerImpl;

#[cfg(feature = "paging")]
use axprocess::handle_page_fault;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(_irq_num: usize) {
        #[cfg(feature = "irq")]
        {
            let guard = kernel_guard::NoPreempt::new();
            axhal::irq::dispatch_irq(_irq_num);
            drop(guard); // rescheduling may occur when preemption is re-enabled.
        }
    }

    #[cfg(feature = "paging")]
    fn handle_page_fault(addr: memory_addr::VirtAddr, flags: page_table::MappingFlags) {
        handle_page_fault(addr, flags);
    }
}
