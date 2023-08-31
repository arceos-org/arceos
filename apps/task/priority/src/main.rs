#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::sync::Arc;
use std::{thread, time};
use std::{vec, vec::Vec};

#[cfg(any(feature = "axstd", target_os = "arceos"))]
use std::os::arceos::api::task::ax_set_current_priority;

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

fn load(n: &u64) -> u64 {
    // time consuming is linear with *n
    let mut sum: u64 = *n;
    for i in 0..*n {
        sum += ((i ^ (i * 3)) ^ (i + *n)) / (i + 1);
    }
    sum
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    #[cfg(feature = "axstd")]
    ax_set_current_priority(-20).ok();

    let data = (0..PAYLOAD_KIND)
        .map(|i| Arc::new(vec![TASK_PARAMS[i].value; TASK_PARAMS[i].data_len]))
        .collect::<Vec<_>>();
    let mut expect: u64 = 0;
    for data_inner in &data {
        expect += data_inner.iter().map(load).sum::<u64>();
    }

    let mut tasks = Vec::with_capacity(PAYLOAD_KIND);
    let start_time = time::Instant::now();
    for i in 0..PAYLOAD_KIND {
        let vec = data[i].clone();
        let data_len = TASK_PARAMS[i].data_len;
        let nice = TASK_PARAMS[i].nice;
        tasks.push(thread::spawn(move || {
            #[cfg(feature = "axstd")]
            ax_set_current_priority(nice).ok();

            let left = 0;
            let right = data_len;
            println!(
                "part {}: {:?} [{}, {})",
                i,
                thread::current().id(),
                left,
                right
            );

            let partial_sum: u64 = vec[left..right].iter().map(load).sum();
            let leave_time = start_time.elapsed().as_millis() as u64;

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

    #[cfg(feature = "axstd")]
    if cfg!(feature = "sched_cfs") && option_env!("AX_SMP") == Some("1") {
        assert!(
            leave_times[0] > leave_times[1]
                && leave_times[1] > leave_times[2]
                && leave_times[2] > leave_times[3]
        );
    }

    assert_eq!(expect, actual);

    println!("Priority tests run OK!");
}
