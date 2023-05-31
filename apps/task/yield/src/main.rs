#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::thread;

const NUM_TASKS: usize = 10;
static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    for i in 0..NUM_TASKS {
        thread::spawn(move || {
            println!("Hello, task {}! id = {:?}", i, thread::current().id());

            #[cfg(not(feature = "preempt"))]
            thread::yield_now();

            let order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            if option_env!("SMP") == Some("1") {
                assert!(order == i); // FIFO scheduler
            }
        });
    }
    println!("Hello, main task!");
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        #[cfg(not(feature = "preempt"))]
        thread::yield_now();
    }
    println!("Task yielding tests run OK!");
}
