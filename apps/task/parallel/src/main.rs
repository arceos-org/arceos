#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::sync::{Mutex, WaitQueue};
use libax::{rand, task};

const NUM_DATA: usize = 1_000_000;
const NUM_TASKS: usize = 16;

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

static MAIN_WQ: WaitQueue = WaitQueue::new();
static RESULTS: Mutex<[u64; NUM_TASKS]> = Mutex::new([0; NUM_TASKS]); // TODO: task join

fn barrier() {
    static BARRIER_WQ: WaitQueue = WaitQueue::new();
    static BARRIER_COUNT: AtomicUsize = AtomicUsize::new(0);
    BARRIER_COUNT.fetch_add(1, Ordering::Relaxed);
    BARRIER_WQ.wait_until(|| BARRIER_COUNT.load(Ordering::Relaxed) == NUM_TASKS);
    BARRIER_WQ.notify_all(true);
}

#[no_mangle]
fn main() {
    let vec = Arc::new(
        (0..NUM_DATA)
            .map(|_| rand::rand_u32() as u64)
            .collect::<Vec<_>>(),
    );
    let expect: u64 = vec.iter().sum();

    for i in 0..NUM_TASKS {
        let vec = vec.clone();
        task::spawn(move || {
            let left = i * (NUM_DATA / NUM_TASKS);
            let right = (left + (NUM_DATA / NUM_TASKS)).min(NUM_DATA);
            println!(
                "part {}: {:?} [{}, {})",
                i,
                task::current().id(),
                left,
                right
            );

            RESULTS.lock()[i] = vec[left..right].iter().sum();

            barrier();

            println!("part {}: {:?} finished", i, task::current().id());
            let n = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            if n == NUM_TASKS - 1 {
                MAIN_WQ.notify_one(true);
            }
        });
    }

    MAIN_WQ.wait();
    println!("main task woken up!");

    let actual = RESULTS.lock().iter().sum();
    println!("sum = {}", actual);
    assert_eq!(expect, actual);

    println!("Parallel summation tests run OK!");
}
