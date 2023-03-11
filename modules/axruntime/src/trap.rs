use spinlock::guard::NoPreempt;

struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(irq_num: usize) {
        let guard = NoPreempt::new();
        axhal::irq::dispatch_irq(irq_num);
        drop(guard); // rescheduling may occur when preemption is re-enabled.
    }
}
