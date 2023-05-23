#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use libax::sync::WaitQueue;
use libax::{rand, thread};

const NUM_DATA: usize = 2_000_000;
const NUM_TASKS: usize = 16;

fn barrier() {
    static BARRIER_WQ: WaitQueue = WaitQueue::new();
    static BARRIER_COUNT: AtomicUsize = AtomicUsize::new(0);
    BARRIER_COUNT.fetch_add(1, Ordering::Relaxed);
    BARRIER_WQ.wait_until(|| BARRIER_COUNT.load(Ordering::Relaxed) == NUM_TASKS);
    BARRIER_WQ.notify_all(true);
}

fn sqrt(n: &u64) -> u64 {
    let mut x = *n;
    loop {
        if x * x <= *n && (x + 1) * (x + 1) > *n {
            return x;
        }
        x = (x + *n / x) / 2;
    }
}

#[no_mangle]
fn main() {
    let vec = Arc::new(
        (0..NUM_DATA)
            .map(|_| rand::rand_u32() as u64)
            .collect::<Vec<_>>(),
    );
    let expect: u64 = vec.iter().map(sqrt).sum();

    // equals to sleep(500ms)
    let timeout = WaitQueue::new().wait_timeout(Duration::from_millis(500));
    assert!(timeout);

    let mut tasks = Vec::with_capacity(NUM_TASKS);
    for i in 0..NUM_TASKS {
        let vec = vec.clone();
        tasks.push(thread::spawn(move || {
            let left = i * (NUM_DATA / NUM_TASKS);
            let right = (left + (NUM_DATA / NUM_TASKS)).min(NUM_DATA);
            println!(
                "part {}: {:?} [{}, {})",
                i,
                thread::current().id(),
                left,
                right
            );

            let partial_sum: u64 = vec[left..right].iter().map(sqrt).sum();
            barrier();

            println!("part {}: {:?} finished", i, thread::current().id());
            partial_sum
        }));
    }

    let actual = tasks.into_iter().map(|t| t.join().unwrap()).sum();
    println!("sum = {}", actual);
    assert_eq!(expect, actual);

    println!("Parallel summation tests run OK!");
}
