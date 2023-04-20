#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use libax::sync::{Mutex, WaitQueue};
use libax::{rand, task};
use libax::task::{sleep, yield_now};

const NUM_DATA: usize = 400000;
const NUM_TASKS: usize = 4;
const INCREMENT_SCALE: u64 = 0;

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

static MAIN_WQ: WaitQueue = WaitQueue::new();
static RESULTS: Mutex<[u64; NUM_TASKS]> = Mutex::new([0; NUM_TASKS]); // TODO: task join
static LEAVE_TIME: Mutex<[u64; NUM_TASKS]> = Mutex::new([0; NUM_TASKS]);

fn barrier() {
    static BARRIER_WQ: WaitQueue = WaitQueue::new();
    static BARRIER_COUNT: AtomicUsize = AtomicUsize::new(0);
    BARRIER_COUNT.fetch_add(1, Ordering::Relaxed);
    BARRIER_WQ.wait_until(|| BARRIER_COUNT.load(Ordering::Relaxed) == NUM_TASKS);
    BARRIER_WQ.notify_all(true);
}

fn load(n: &u64) -> u64 {
    // 一个高耗时负载，运行 1000+n 次
    let mut sum : u64 = *n;
    for i in 0..100 + INCREMENT_SCALE * (*n) {
        sum = sum + ((i * i) ^ (i + *n)) / (i + 1);
    }
    yield_now();
    sum
}

#[no_mangle]
fn main() {
    // 短任务，测试调度速度
    let vec = Arc::new(
        (0..NUM_DATA)
            .map(|idx| idx as u64)
            .collect::<Vec<_>>(),
    );
    let expect: u64 = vec.iter().map(load).sum();

    let timeout = MAIN_WQ.wait_timeout(Duration::from_millis(500));
    assert!(timeout);

    for i in 0..NUM_TASKS {
        let vec = vec.clone();
        task::spawn(move || {
            let start_time = libax::time::Instant::now();
            let left = i * (NUM_DATA / NUM_TASKS);
            let right = (left + (NUM_DATA / NUM_TASKS)).min(NUM_DATA);
            println!(
                "part {}: {:?} [{}, {})",
                i,
                task::current().id(),
                left,
                right
            );

            RESULTS.lock()[i] = vec[left..right].iter().map(load).sum();
            LEAVE_TIME.lock()[i] = start_time.elapsed().as_millis() as u64;

            barrier();

            println!("part {}: {:?} finished", i, task::current().id());
            let n = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            if n == NUM_TASKS - 1 {
                MAIN_WQ.notify_one(true);
            }
        }, 0);
    }
    let timeout = MAIN_WQ.wait_timeout(Duration::from_millis(20000));
    println!("main task woken up! timeout={}", timeout);

    let actual = RESULTS.lock().iter().sum();
    let binding = LEAVE_TIME.lock();
    for i in 0..NUM_TASKS {
        println!("leave time id {} = {}ms", i, binding[i]);
    }
    drop(binding);
    let binding = LEAVE_TIME.lock();
    let max_leave_time = binding.iter().max();
    println!("maximum leave time = {}ms", max_leave_time.unwrap());
    drop(binding);
    println!("sum = {}", actual);
    let binding = LEAVE_TIME.lock();
    let min_leave_time = binding.iter().min();
    println!("minimum leave time = {}ms", min_leave_time.unwrap());
    assert_eq!(expect, actual);

    println!("Parallel summation tests run OK!");
}
