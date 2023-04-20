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

const NUM_DATA_RUNTIME_1: usize = 1;
const NUM_DATA_RUNTIME_2: usize = 2;
const NUM_DATA_RUNTIME_3: usize = 1;
const NUM_DATA_PERIOD_1: usize = 3;
const NUM_DATA_PERIOD_2: usize = 5;
const NUM_DATA_PERIOD_3: usize = 6;
const NUM_RUN_TIMES    : usize = 1000; 
const PAYLOAD_KIND     : usize = 3;


static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

static MAIN_WQ: WaitQueue = WaitQueue::new();
static RESULTS: Mutex<[u64; PAYLOAD_KIND]> = Mutex::new([0; PAYLOAD_KIND]); // TODO: task join
static LEAVE_TIME: Mutex<[u64; PAYLOAD_KIND]> = Mutex::new([0; PAYLOAD_KIND]);

fn barrier() {
    static BARRIER_WQ: WaitQueue = WaitQueue::new();
    static BARRIER_COUNT: AtomicUsize = AtomicUsize::new(0);
    BARRIER_COUNT.fetch_add(1, Ordering::Relaxed);
    BARRIER_WQ.wait_until(|| BARRIER_COUNT.load(Ordering::Relaxed) == PAYLOAD_KIND);
    BARRIER_WQ.notify_all(true);
}

fn load(runtime: usize, sleeptime: usize) -> u64 {
    // 一个高耗时负载
    let mut sum : u64 = runtime as u64;
    for k in 0..runtime {
        println!("runtime = {}, sleeptime = {}", runtime, sleeptime);
        for i in 0..16000000 { // 每 runtime ~50ms
            sum = sum + ((i * i) ^ (i + runtime as u64)) / (i + 1);
        }
        if k + 1 != runtime {
            yield_now();
        }
    }
    sleep(Duration::from_millis(50 * sleeptime as u64));
    sum
}

#[no_mangle]
fn main() {
    let timeout = MAIN_WQ.wait_timeout(Duration::from_millis(500));
    assert!(timeout);

    for ii in 0..PAYLOAD_KIND {
        let i = PAYLOAD_KIND - 1 - ii; 
        let datalen: usize;
        let sleeplen: usize;
        if i == 0 {
            datalen = NUM_DATA_RUNTIME_1;
            sleeplen = NUM_DATA_PERIOD_1 - NUM_DATA_RUNTIME_1;
        } else if i == 1 {
            datalen = NUM_DATA_RUNTIME_2;
            sleeplen = NUM_DATA_PERIOD_2 - NUM_DATA_RUNTIME_2;
        } else if i == 2 {
            datalen = NUM_DATA_RUNTIME_3;
            sleeplen = NUM_DATA_PERIOD_3 - NUM_DATA_RUNTIME_3;
        } else {
            datalen = 0;
            sleeplen = 0;
        }
        task::spawn(move || {
            let start_time = libax::time::Instant::now();
            let left = 0;
            let right = NUM_RUN_TIMES;
            println!(
                "part {}: {:?} [{}, {})",
                i,
                task::current().id(),
                left,
                right
            );
            let mut tmp: u64 = 0;
            for i in left..right {
                tmp += load(datalen, sleeplen);
            }
            RESULTS.lock()[i] = tmp;
            LEAVE_TIME.lock()[i] = start_time.elapsed().as_millis() as u64;

            //barrier();

            println!("part {}: {:?} finished", i, task::current().id());
            let n = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            if i == PAYLOAD_KIND - 1 { // 注意这里只要高耗时进程结束就退出
                MAIN_WQ.notify_one(true);
            }
        }, datalen, datalen + sleeplen);
    }

    let timeout = MAIN_WQ.wait_timeout(Duration::from_millis(20000));
    println!("main task woken up! timeout={}", timeout);

    //let actual = RESULTS.lock().iter().sum();
    let binding = LEAVE_TIME.lock();
    let long_task_leave_time = binding[PAYLOAD_KIND - 1];
    println!("long task leave time = {}ms", long_task_leave_time);
    drop(binding);
    //println!("sum = {}", actual);
    //assert_eq!(expect, actual);

    println!("Parallel summation tests run OK!");
}
