#![no_std]
#![no_main]

#[macro_use]
extern crate axruntime;

use core::sync::atomic::{AtomicUsize, Ordering};

const NUM_TASKS: usize = 10;
static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    for i in 0..NUM_TASKS {
        axtask::spawn(move || {
            println!("Hello, task {}! id = {:?}", i, axtask::current().id());
            axtask::yield_now();
            let order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            assert!(order == i); // FIFO scheduler
        });
    }
    println!("Hello, main task!");
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        axtask::yield_now();
    }
    println!("Multitask tests run OK!");
}
