extern crate alloc;

use lazyinit::LazyInit;

use kspin::SpinNoIrq;

use axhal::cpu::this_cpu_id;
use axhal::irq::IPI_IRQ_NUM;

mod queue;

use queue::IPIEventQueue;

pub use queue::IPIEventFn;

#[percpu::def_percpu]
static IPI_MSG_QUEUE: LazyInit<SpinNoIrq<IPIEventQueue<IPIEventFn>>> = LazyInit::new();

pub fn init() {
    IPI_MSG_QUEUE.with_current(|ipi_queue| {
        ipi_queue.init_once(SpinNoIrq::new(IPIEventQueue::default()));
    });
    axhal::irq::register_handler(IPI_IRQ_NUM, ipi_handler);
}

pub fn send_ipi_event(target_cpu: usize, event: IPIEventFn) {
    unsafe { IPI_MSG_QUEUE.remote_ref_raw(target_cpu) }
        .lock()
        .push(this_cpu_id(), event);
}

fn ipi_handler() {
    while let Some(event) = unsafe { IPI_MSG_QUEUE.current_ref_mut_raw() }
        .lock()
        .pop_one()
    {
        event();
    }
}
