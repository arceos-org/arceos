struct TrapHandlerImpl;

#[cfg(feature = "paging")]
use axhal::{arch::TrapFrame, mem::VirtAddr, paging::MappingFlags};

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
    fn handle_page_fault(_addr: VirtAddr, _flags: MappingFlags, _tf: &mut TrapFrame) {
        unimplemented!();
    }
}
