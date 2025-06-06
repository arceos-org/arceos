#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use axasync::executor::spawner;
use axasync::executor::yield_now;
use axasync::time::Timer;
use core::hint::black_box;
use std::thread::{self, sleep};
use std::time::Duration;

fn busy_work(iters: u64) -> u64 {
    let mut total = 0;
    for _ in 0..iters {
        total = black_box(total + 1);
        total = black_box(total * 3 / 2);
    }
    black_box(total)
}

async fn async_busy_work(iters: u64) -> u64 {
    let mut total = 0;
    for _ in 0..iters {
        total = black_box(total + 1);
        total = black_box(total * 3 / 2);
        yield_now().await;
    }
    black_box(total)
}

macro_rules! task_loop {
    (
        task_type: $task_type:literal,
        id: $id:expr,
        sleep: $sleep_fn:expr,
        // millis restriction
        millis: $millis:expr,
        $(is_await: $await_tt:tt,)?
        busy_work: $busy_work:expr,
        busy_iters: $busy_iters:expr,
    ) => {
        use std::time::{Instant,Duration};
        use log;
        use core::hint::black_box;

        let mut cnt = 0;
        let mut last_report = Instant::now();
        let millis = Duration::from_millis($millis);

        loop {
            let iter_start = Instant::now();
            cnt += 1;

            if $busy_iters > 0 {
                let busy_start = Instant::now();
                let _res = black_box($busy_work($busy_iters));
                let busy_dur = Instant::now() - busy_start;
                log::info!(
                    "{} {}: duration {}/ns",
                    $task_type,
                    $id,
                    busy_dur.as_nanos()
                )
            }

            $sleep_fn($millis)$(.$await_tt)?;

            let iter_end = Instant::now();
            let iter_dur = iter_end - iter_start;
            let full_dur = iter_end - last_report;

            log::info!(
                "{} {}: Iteration {}, expected {}/ns, actual {}/ns, full {}/ns",
                $task_type,
                $id,
                cnt,
                millis.as_nanos(),
                iter_dur.as_nanos(),
                full_dur.as_nanos(),
            );

            last_report = iter_end;
        }
    };
}

#[embassy_executor::task(pool_size = 5)]
async fn async_tick(id: u64, millis: u64, busy_iters: u64) {
    task_loop! {
        task_type: "ASYNC_TASK_REPORT",
        id:id,
        sleep: |millis| async move {Timer::after_millis(millis).await},
        millis:millis,
        is_await: await,
        busy_work: async_busy_work,
        busy_iters:busy_iters,
    }
}

fn thread_tick(id: u64, millis: u64, busy_iters: u64) {
    task_loop! {
        task_type: "NATIVE_THREAD_REPORT",
        id:id,
        sleep: |millis| sleep(Duration::from_millis(millis)),
        millis:millis,
        busy_work: busy_work,
        busy_iters:busy_iters,
    }
}

const NUM_THREADS: u64 = 5;
const NUM_TASKS: u64 = 5;
// const NUM_ITERS_THREADS: u64 = 0;
// const NUM_ITERS_THREADS: u64 = 1_000;
// const NUM_ITERS_THREADS: u64 = 1_000_000;
const NUM_ITERS_THREADS: u64 = 100_000_000;
// const NUM_ITERS_TASKS: u64 = 0;
// const NUM_ITERS_TASKS: u64 = 1_000;
// const NUM_ITERS_TASKS: u64 = 1_000_000;
const NUM_ITERS_TASKS: u64 = 100_000_000;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    log::info!("Embassy Test");
    for i in 1..NUM_THREADS {
        thread::spawn(move || {
            thread_tick(i, i * 1000, NUM_ITERS_THREADS);
        });
    }

    for i in 1..NUM_TASKS {
        spawner()
            .spawn(async_tick(i, i * 1000, NUM_ITERS_TASKS))
            .unwrap();
    }
    // Avoid shut down immediately
    sleep(Duration::from_millis(1000 * 15 as u64));
}
