#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use libax::time::Duration;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::sync::{Mutex, WaitQueue};
use libax::thread;

struct TaskParam {
    data_len: usize,
    value: u64,
    nice: isize,
}

const PAYLOAD_KIND: usize = 200;

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

static MAIN_WQ: WaitQueue = WaitQueue::new();
static RESULTS: Mutex<[u64; PAYLOAD_KIND]> = Mutex::new([0; PAYLOAD_KIND]); // TODO: task join
static LEAVE_TIME: Mutex<[u64; PAYLOAD_KIND]> = Mutex::new([0; PAYLOAD_KIND]);

fn load(n: &u64) -> u64 {
    // time consuming is linear with *n
    let mut sum: u64 = *n;
    for i in 0..*n {
        sum += ((i ^ (i * 3)) ^ (i + *n)) / (i + 1);
    }
    sum
}

#[no_mangle]
fn main() {
    thread::set_priority(-20);
    let mut expect: u64 = 0;
    
    let timeout = WaitQueue::new().wait_timeout(Duration::from_millis(500));

    let start_time = libax::time::Instant::now();

    let wait_time = 3000u64;

    for i in 0..PAYLOAD_KIND {
        thread::spawn(move || {
            thread::set_priority(19);
            println!(
                "part {}: {:?}",
                i,
                thread::current().id()
            );

            let timeout = WaitQueue::new().wait_timeout(Duration::from_millis(wait_time - start_time.elapsed().as_millis() as u64));
            LEAVE_TIME.lock()[i] = start_time.elapsed().as_millis() as u64;

            println!("part {}: {:?} finished", i, thread::current().id());
            let n = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            if n == PAYLOAD_KIND - 1 {
                MAIN_WQ.notify_one(true);
            }
        });
    }

    MAIN_WQ.wait();

    let actual = RESULTS.lock().iter().sum();
    println!("sum = {}", actual);
    let level_times = LEAVE_TIME.lock();
    println!("leave time:");
    let mut mx = 0;
    for i in 0..PAYLOAD_KIND {
        println!("task {} = {}ms", i, level_times[i] - wait_time);
        if level_times[i] - wait_time > mx {
            mx = level_times[i] - wait_time;
        }
    }
    println!("leave time max: {}", mx);

    assert_eq!(expect, actual);

    println!("Priority tests run OK!");
}
