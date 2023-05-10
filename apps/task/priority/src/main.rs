#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use libax::sync::{Mutex, WaitQueue};
use libax::task;

struct TaskParam {
    data_len: usize,
    value: u64,
    nice: isize,
}

const TASK_PARAMS: &[TaskParam] = &[
    // four short tasks
    TaskParam {
        data_len: 40,
        value: 1000000,
        nice: 19,
    },
    TaskParam {
        data_len: 40,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 40,
        value: 1000000,
        nice: 0,
    },
    TaskParam {
        data_len: 40,
        value: 1000000,
        nice: -10,
    },
    // one long task
    TaskParam {
        data_len: 4,
        value: 10000000,
        nice: 0,
    },
];

const PAYLOAD_KIND: usize = 5;

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
    task::set_priority(-20);
    let data = (0..PAYLOAD_KIND)
        .map(|i| Arc::new(vec![TASK_PARAMS[i].value; TASK_PARAMS[i].data_len]))
        .collect::<Vec<_>>();
    let mut expect: u64 = 0;
    for data_inner in &data {
        expect += data_inner.iter().map(load).sum::<u64>();
    }
    let start_time = libax::time::Instant::now();
    for i in 0..PAYLOAD_KIND {
        let vec = data[i].clone();
        let data_len = TASK_PARAMS[i].data_len;
        let nice = TASK_PARAMS[i].nice;
        task::spawn(move || {
            let left = 0;
            let right = data_len;
            task::set_priority(nice);
            println!(
                "part {}: {:?} [{}, {})",
                i,
                task::current().id(),
                left,
                right
            );

            RESULTS.lock()[i] = vec[left..right].iter().map(load).sum();
            LEAVE_TIME.lock()[i] = start_time.elapsed().as_millis() as u64;

            println!("part {}: {:?} finished", i, task::current().id());
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
    for i in 0..PAYLOAD_KIND {
        println!("task {} = {}ms", i, level_times[i]);
    }

    if cfg!(feature = "sched_cfs") && option_env!("SMP") == Some("1") {
        assert!(
            level_times[0] > level_times[1]
                && level_times[1] > level_times[2]
                && level_times[2] > level_times[3]
        );
    }

    assert_eq!(expect, actual);

    println!("Priority tests run OK!");
}
