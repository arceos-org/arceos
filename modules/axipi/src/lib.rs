//! [ArceOS](https://github.com/arceos-org/arceos) Inter-Processor Interrupt (IPI) primitives.

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate log;
extern crate alloc;

use lazyinit::LazyInit;

use kspin::SpinNoIrq;

use axhal::cpu::this_cpu_id;
use axhal::irq::IPI_IRQ_NUM;

mod event;
mod queue;

use queue::IPIEventQueue;
pub use event::*;

#[percpu::def_percpu]
static IPI_EVENT_QUEUE: LazyInit<SpinNoIrq<IPIEventQueue>> = LazyInit::new();

/// Initialize the per-CPU IPI event queue.
pub fn init() {
    IPI_EVENT_QUEUE.with_current(|ipi_queue| {
        ipi_queue.init_once(SpinNoIrq::new(IPIEventQueue::default()));
    });
}

/// Sends an IPI event to the processor(s) specified by `dest_cpu`.
pub fn send_ipi_event_to_one<T: Into<Callback>>(dest_cpu: usize, callback: T) {
    warn!("Send IPI event to CPU {}", dest_cpu);

    unsafe { IPI_EVENT_QUEUE.remote_ref_raw(dest_cpu) }
        .lock()
        .push(this_cpu_id(), callback.into());
    axhal::irq::send_sgi_one(dest_cpu, IPI_IRQ_NUM);
}

/// Sends an IPI event to all processors except the current one.
pub fn send_ipi_event_to_all<T: Into<MulticastCallback>>(callback: T) {
    let current_cpu_id = this_cpu_id();
    let callback = callback.into();
    for cpu_id in 0..axconfig::SMP {
        if cpu_id != current_cpu_id {
            unsafe { IPI_EVENT_QUEUE.remote_ref_raw(cpu_id) }
                .lock()
                .push(current_cpu_id, callback.clone().into_unicast());
        }
    }
    axhal::irq::send_sgi_all(IPI_IRQ_NUM);
}

pub fn ipi_handler() {
    while let Some((src_cpu_id, callback)) = unsafe { IPI_EVENT_QUEUE.current_ref_mut_raw() }
        .lock()
        .pop_one()
    {
        warn!("Received IPI event from CPU {}", src_cpu_id);
        callback.call();
    }
}
