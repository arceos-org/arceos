#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

const NUM_TASKS: usize = 10;
static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    for i in 0..NUM_TASKS {
        thread::spawn(move || {
            println!("Hello, task {}! id = {:?}", i, thread::current().id());

            #[cfg(all(not(feature = "sched_rr"), not(feature = "sched_cfs")))]
            thread::yield_now();

            let _order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            #[cfg(not(feature = "sched_cfs"))]
            if option_env!("AX_SMP") == Some("1") {
                assert!(_order == i); // FIFO scheduler
            }
        });
    }
    println!("Hello, main task!");
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        #[cfg(all(not(feature = "sched_rr"), not(feature = "sched_cfs")))]
        thread::yield_now();
    }
    println!("Task yielding tests run OK!");
}
