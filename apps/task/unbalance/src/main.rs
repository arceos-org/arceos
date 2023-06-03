#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::sync::{Mutex, WaitQueue};
use libax::thread;
use libax::time::Duration;

struct TaskParam {
    data_len: usize,
    value: u64,
    nice: isize,
}

const TASK_PARAMS: &[TaskParam] = &[
    TaskParam {
        data_len: 100,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 2,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 3,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 4,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 6,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 7,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 8,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 9,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 10,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 15,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 20,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 25,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 30,
        value: 10000000,
        nice: 10,
    },
    TaskParam {
        data_len: 30,
        value: 10000000,
        nice: 10,
    },
];

const PAYLOAD_KIND: usize = 25;

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
    let data = (0..PAYLOAD_KIND)
        .map(|i| Arc::new(vec![TASK_PARAMS[i].value; TASK_PARAMS[i].data_len]))
        .collect::<Vec<_>>();
    let mut expect: u64 = 0;
    for data_inner in &data {
        expect += data_inner.iter().map(load).sum::<u64>();
    }

    let timeout = WaitQueue::new().wait_timeout(Duration::from_millis(500));

    let start_time = libax::time::Instant::now();

    let wait_time = 3000u64;

    for ii in 0..PAYLOAD_KIND {
        let i = PAYLOAD_KIND - 1 - ii;
        let vec = data[i].clone();
        let data_len = TASK_PARAMS[i].data_len;
        let nice = TASK_PARAMS[i].nice;
        thread::spawn(move || {
            let left = 0;
            let right = data_len;
            thread::set_priority(nice);
            println!(
                "part {}: {:?} [{}, {})",
                i,
                thread::current().id(),
                left,
                right
            );

            let timeout = WaitQueue::new().wait_timeout(Duration::from_millis(
                wait_time - start_time.elapsed().as_millis() as u64,
            ));
            RESULTS.lock()[i] = vec[left..right].iter().map(load).sum();
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

    println!("Unbalance tests run OK!");
}
