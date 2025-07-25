//! [ArceOS](https://github.com/arceos-org/arceos) Inter-Processor Interrupt (IPI) primitives.

#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate log;
extern crate alloc;

use lazyinit::LazyInit;

use kspin::SpinNoIrq;

use axhal::irq::{IpiTarget, IPI_IRQ};
use axhal::percpu::this_cpu_id;

mod event;
mod queue;

pub use event::*;
use queue::IPIEventQueue;

#[percpu::def_percpu]
static IPI_EVENT_QUEUE: LazyInit<SpinNoIrq<IPIEventQueue>> = LazyInit::new();

/// Initialize the per-CPU IPI event queue.
pub fn init() {
    IPI_EVENT_QUEUE.with_current(|ipi_queue| {
        ipi_queue.init_once(SpinNoIrq::new(IPIEventQueue::default()));
    });
}

/// Sends an IPI event to the processor(s) specified by `dest_cpu`.
pub fn send_ipi_one<T: Into<Callback>>(dest_cpu: usize, callback: T) {
    info!("Send IPI event to CPU {}", dest_cpu);

    unsafe { IPI_EVENT_QUEUE.remote_ref_raw(dest_cpu) }
        .lock()
        .push(this_cpu_id(), callback.into());
    axhal::irq::send_ipi(IPI_IRQ, None, Some(dest_cpu), None, IpiTarget::Specific);
}

/// Sends an IPI event to all processors except the current one.
pub fn send_ipi_allothers<T: Into<MulticastCallback>>(callback: T) {
    info!("Send IPI event to all other CPUs");
    let current_cpu_id = this_cpu_id();
    let cpu_num = axconfig::plat::CPU_NUM;
    let callback = callback.into();
    for cpu_id in 0..cpu_num {
        if cpu_id != current_cpu_id {
            unsafe { IPI_EVENT_QUEUE.remote_ref_raw(cpu_id) }
                .lock()
                .push(current_cpu_id, callback.clone().into_unicast());
        }
    }
    axhal::irq::send_ipi(
        IPI_IRQ,
        Some(current_cpu_id),
        None,
        Some(cpu_num),
        IpiTarget::AllOthers,
    );
}

/// The handler for IPI events. It retrieves the events from the queue and calls the corresponding callbacks.
pub fn ipi_handler() {
    while let Some((src_cpu_id, callback)) = unsafe { IPI_EVENT_QUEUE.current_ref_mut_raw() }
        .lock()
        .pop_one()
    {
        debug!("Received IPI event from CPU {}", src_cpu_id);
        callback.call();
    }
}
