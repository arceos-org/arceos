#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

extern crate alloc;

use alloc::sync::Arc;
use axasync::executor::{PrioFuture, spawner, yield_now};
use axasync::time::Timer;
use core::hint::black_box;
use core::sync::atomic::AtomicUsize;
use std::thread::{self, sleep};
use std::time::Duration;

macro_rules! work_loop {
    (
        task_type: $task_type:literal,
        id: $id:expr,
        expected: $millis:expr,
        volume: $volume:expr,
        busy: {$($busy_tt:tt)*},
        sleep: {$($sleep_tt:tt)*},
        $(output: {$($output_tt:tt)*},)?
    ) => {
        use std::time::{Instant,Duration};
        use log;
        use core::hint::black_box;

        let mut cnt = 0;
        let mut last_report = Instant::now();
        let expected = Duration::from_millis($millis);
        loop {
            let iter_start = Instant::now();
            cnt += 1;

            $($busy_tt)*
            $($sleep_tt)*
            $($($output_tt)*)?

            let iter_end = Instant::now();
            let iter_dur = iter_end - iter_start;
            let full_dur = iter_end - last_report;

            log::info!(
                "{} {}: volume {}, works {}, times {}/s, iters {}, expected {}/ns, actual {}/ns, full {}/ns",
                $task_type,
                $id,
                $volume,
                NUM_ITERS,
                TEST_SECS,
                cnt,
                expected.as_nanos(),
                iter_dur.as_nanos(),
                full_dur.as_nanos(),
            );

            last_report = iter_end;
        }
    };
}

macro_rules! prio_task {
    (
        fn_name: $fn_name:ident,
        task_type: $task_type:literal,
        prios: $prios:expr,
        pool_size: $pool_size:expr,
        atomic,
    ) => {
        #[axasync::executor::task(pool_size = $pool_size)]
        async fn $fn_name(id: u64, millis: u64, busy_iters: u64, iters: Arc<AtomicUsize>) {
            work_loop! {
                task_type: $task_type,
                id: id,
                expected: millis,
                volume: $pool_size,
                busy: {
                    if busy_iters > 0 {
                        let _res = black_box(PrioFuture::new(async_busy_work(busy_iters), $prios).await);
                    }
                },
                sleep: {
                    PrioFuture::new(Timer::after_millis(millis),$prios).await;
                },
                output: {
                    iters.fetch_add(1,core::sync::atomic::Ordering::SeqCst);
                },
            }
        }
    };
    (
        fn_name: $fn_name:ident,
        task_type: $task_type:literal,
        prios: $prios:expr,
        pool_size: $pool_size:expr,
    ) => (
        #[axasync::executor::task(pool_size = $pool_size)]
        async fn $fn_name(id: u64, millis: u64, busy_iters: u64) {
            work_loop! {
                task_type: $task_type,
                id: id,
                expected: millis,
                volume: $pool_size,
                busy: {
                    if busy_iters > 0 {
                        let _res = black_box(PrioFuture::new(async_busy_work(busy_iters), $prios).await);
                    }
                },
                sleep: {
                    PrioFuture::new(Timer::after_millis(millis),$prios).await;
                },
            }
        }
    )
}

const HIGH_PRIOS: u8 = 1;
const LOW_PRIOS: u8 = 3;

#[cfg(feature = "async-test")]
prio_task! {
    fn_name: prio_tick_high,
    task_type: "ASYNC_TASK_REPORT_HIGH",
    prios: HIGH_PRIOS,
    pool_size: NUM_HIGH_TASKS as usize,
}

#[cfg(feature = "async-test")]
prio_task! {
    fn_name: prio_tick_low,
    task_type: "ASYNC_TASK_REPORT_LOW",
    prios: LOW_PRIOS,
    pool_size: NUM_LOW_TASKS as usize,
}

#[cfg(feature = "async-test")]
prio_task! {
    fn_name: prio_add_high,
    task_type: "ASYNC_TASK_REPORT_HIGH",
    prios: HIGH_PRIOS,
    pool_size: NUM_HIGH_TASKS as usize,
    atomic,
}

#[cfg(feature = "async-test")]
prio_task! {
    fn_name: prio_add_low,
    task_type: "ASYNC_TASK_REPORT_LOW",
    prios: LOW_PRIOS,
    pool_size: NUM_LOW_TASKS as usize,
    atomic,
}

#[cfg(feature = "thread-test")]
fn thread_tick(id: u64, millis: u64, busy_iters: u64) {
    work_loop! {
        task_type: "NATIVE_THREAD_REPORT",
        id:id,
        expected: millis,
        volume: NUM_THREADS,
        busy: {
            if busy_iters > 0 {
                let _res = black_box(busy_work(busy_iters));
            }
        },
        sleep: {
            sleep(Duration::from_millis(millis));
        },
        output: {},
    }
}

#[cfg(feature = "thread-test")]
fn thread_add(id: u64, millis: u64, busy_iters: u64, iters: Arc<AtomicUsize>) {
    work_loop! {
        task_type: "NATIVE_THREAD_ATOMIC_REPORT",
        id:id,
        expected: millis,
        volume: NUM_THREADS,
        busy: {
            if busy_iters > 0 {
                let _res = black_box(busy_work(busy_iters));
            }
        },
        sleep: {
            sleep(Duration::from_millis(millis));
        },
        output: {
            iters.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        },
    }
}

fn busy_work(iters: u64) -> u64 {
    let mut total = 0;
    for _ in 0..iters {
        total = black_box(total + 1);
        thread::yield_now();
    }
    black_box(total)
}

async fn async_busy_work(iters: u64) -> u64 {
    let mut total = 0;
    for _ in 0..iters {
        total = black_box(total + 1);
        yield_now().await;
    }
    black_box(total)
}

// const NUM_THREADS: u64 = 30;
const NUM_THREADS: u64 = 50;
// const NUM_THREADS: u64 = 100;
// const NUM_THREADS: u64 = 125;
// const NUM_THREADS: u64 = 150;
// const NUM_THREADS: u64 = 175;
//
const NUM_HIGH_TASKS: u64 = 15;
// const NUM_HIGH_TASKS: u64 = 25;
// const NUM_HIGH_TASKS: u64 = 30;
// const NUM_HIGH_TASKS: u64 = 50;
// const NUM_HIGH_TASKS: u64 = 75;
// const NUM_HIGH_TASKS: u64 = 100;
// const NUM_HIGH_TASKS: u64 = 200;
//
const NUM_LOW_TASKS: u64 = 15;
// const NUM_LOW_TASKS: u64 = 25;
// const NUM_LOW_TASKS: u64 = 30;
// const NUM_LOW_TASKS: u64 = 50;
// const NUM_LOW_TASKS: u64 = 75;
// const NUM_LOW_TASKS: u64 = 100;
// const NUM_LOW_TASKS: u64 = 200;
//

// const NUM_ITERS: u64 = 100;
// The num iters for volume delay
// const NUM_ITERS: u64 = 1000;
// const NUM_ITERS: u64 = 1_000_0;
// const NUM_ITERS: u64 = 1_000_00;
const NUM_ITERS: u64 = 10;
// const NUM_ITERS: u64 = 1_000_000_0;
// const NUM_ITERS: u64 = 100;
const TEST_SECS: u64 = 15;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    log::info!("Starting Test");
    let th_iters = Arc::new(AtomicUsize::new(0));
    let high_prio_iters = Arc::new(AtomicUsize::new(0));
    let low_prio_iters = Arc::new(AtomicUsize::new(0));

    #[cfg(feature = "thread-test")]
    for i in 1..NUM_THREADS {
        #[cfg(feature = "atomic-sum")]
        let th_iters = th_iters.clone();

        thread::spawn(move || {
            #[cfg(feature = "atomic-sum")]
            {
                thread_add(i, (i % 20 + 1), NUM_ITERS, th_iters);
            }
            #[cfg(feature = "iter-delay")]
            {
                thread_tick(i, (i % 20 + 1) * 1000, NUM_ITERS);
            }
        });
    }

    #[cfg(feature = "async-test")]
    for i in 1..NUM_LOW_TASKS {
        if i <= NUM_HIGH_TASKS {
            #[cfg(feature = "atomic-sum")]
            {
                let high_prio_iters = high_prio_iters.clone();
                spawner().must_spawn(prio_add_high(i, (i % 20 + 1), NUM_ITERS, high_prio_iters));
            }
            #[cfg(feature = "iter-delay")]
            {
                spawner()
                    .spawn(prio_tick_high(i, (i % 20 + 1) * 1000, NUM_ITERS))
                    .unwrap();
            }
        }
        #[cfg(feature = "atomic-sum")]
        {
            let low_prio_iters = low_prio_iters.clone();
            spawner().must_spawn(prio_add_low(
                i + NUM_HIGH_TASKS,
                (i % 20 + 1),
                NUM_ITERS,
                low_prio_iters,
            ));
        }
        #[cfg(feature = "iter-delay")]
        {
            spawner()
                .spawn(prio_tick_low(
                    i + NUM_HIGH_TASKS,
                    (i % 20 + 1) * 1000,
                    NUM_ITERS,
                ))
                .unwrap();
        }
    }
    // Avoid shut down immediately
    sleep(Duration::from_secs(TEST_SECS));

    #[cfg(feature = "atomic-sum")]
    {
        let th_out = th_iters.load(core::sync::atomic::Ordering::Relaxed);
        let high_out = high_prio_iters.load(core::sync::atomic::Ordering::Relaxed);
        let low_out = low_prio_iters.load(core::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "thread-test")]
        log::info!(
            "NATIVE_THREAD_REPORT: volume: {}, time: {}/s, works: {}, sum: {}, sum/s: {}",
            NUM_THREADS,
            TEST_SECS,
            NUM_ITERS,
            th_out,
            th_out as u64 / TEST_SECS
        );
        #[cfg(feature = "async-test")]
        {
            log::info!(
                "ASYNC_TASK_REPORT_HIGH: volume: {}, time: {}/s, works: {}, sum: {}, sum/s: {}",
                NUM_HIGH_TASKS,
                TEST_SECS,
                NUM_ITERS,
                high_out,
                high_out as u64 / TEST_SECS
            );
            log::info!(
                "ASYNC_TASK_REPORT_LOW: volume: {}, time: {}/s, works: {}, sum: {}, sum/s: {}",
                NUM_LOW_TASKS,
                TEST_SECS,
                NUM_ITERS,
                low_out,
                low_out as u64 / TEST_SECS
            );
        }
    }
}
