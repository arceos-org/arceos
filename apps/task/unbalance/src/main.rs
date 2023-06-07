#![no_std]
#![no_main]

#[macro_use]
extern crate libax;
extern crate alloc;

use alloc::sync::Arc;
use alloc::vec::Vec;
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
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 2,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 3,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 4,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 5,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 6,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 7,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 8,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 9,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 10,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 15,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 20,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 25,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 30,
        value: 1000000,
        nice: 10,
    },
    TaskParam {
        data_len: 30,
        value: 1000000,
        nice: 10,
    },
];

const PAYLOAD_KIND: usize = 25;

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

    let sleep_dur = Duration::new(0, 300000000);
    let start_time = libax::time::Instant::now();
    let wakeup_time = start_time.as_duration() + sleep_dur;

    let mut tasks = Vec::with_capacity(PAYLOAD_KIND);
    for ii in 0..PAYLOAD_KIND {
        let i = PAYLOAD_KIND - 1 - ii;
        let vec = data[i].clone();
        let data_len = TASK_PARAMS[i].data_len;
        let nice = TASK_PARAMS[i].nice;
        tasks.push(thread::spawn(move || {
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

            thread::sleep_until(wakeup_time);
            let partial_sum: u64 = vec[left..right].iter().map(load).sum();
            let leave_time = (start_time.elapsed() - sleep_dur).as_millis() as u64;

            println!("part {}: {:?} finished", i, thread::current().id());
            (partial_sum, leave_time)
        }));
    }

    let (results, leave_times): (Vec<_>, Vec<_>) =
        tasks.into_iter().map(|t| t.join().unwrap()).unzip();
    let actual = results.iter().sum();

    println!("sum = {}", actual);
    println!("leave time:");
    for (i, time) in leave_times.iter().enumerate() {
        println!("task {} = {}ms", i, time);
    }
    println!("max leave time: {}ms", leave_times.iter().max().unwrap());

    assert_eq!(expect, actual);

    println!("Unbalance tests run OK!");
}
