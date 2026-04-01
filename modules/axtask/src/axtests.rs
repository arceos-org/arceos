
use alloc::{format, vec::Vec};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use axtest::{ax_assert, ax_assert_eq, def_test};
use kspin::SpinNoIrq;

use crate::{WaitQueue, api as axtask, current};

static SERIAL: SpinNoIrq<()> = SpinNoIrq::new(());

#[def_test]
fn test_sched_fifo() {
    let _lock = SERIAL.lock();

    const NUM_TASKS: usize = 10;
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);
    FINISHED_TASKS.store(0, Ordering::Release);

    for i in 0..NUM_TASKS {
        axtask::spawn_raw(
            move || {
                let _ = i;
                axtask::yield_now();
                FINISHED_TASKS.fetch_add(1, Ordering::Release);
            },
            format!("T{}", i),
            0x1000,
        );
    }

    while FINISHED_TASKS.load(Ordering::Acquire) < NUM_TASKS {
        axtask::yield_now();
    }
}

#[def_test]
fn test_fp_state_switch() {
    let _lock = SERIAL.lock();

    const NUM_TASKS: usize = 5;
    const FLOATS: [f64; NUM_TASKS] = [
        3.141592653589793,
        2.718281828459045,
        -1.4142135623730951,
        0.0,
        0.618033988749895,
    ];
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);
    static HAS_ERROR: AtomicBool = AtomicBool::new(false);
    FINISHED_TASKS.store(0, Ordering::Release);
    HAS_ERROR.store(false, Ordering::Release);

    for (i, float) in FLOATS.iter().copied().enumerate() {
        axtask::spawn(move || {
            let mut value = float + i as f64;
            axtask::yield_now();
            value -= i as f64;

            if (value - float).abs() >= 1e-9 {
                HAS_ERROR.store(true, Ordering::Release);
            }
            FINISHED_TASKS.fetch_add(1, Ordering::Release);
        });
    }

    while FINISHED_TASKS.load(Ordering::Acquire) < NUM_TASKS {
        axtask::yield_now();
    }
    ax_assert!(!HAS_ERROR.load(Ordering::Acquire));
}

#[def_test]
fn test_wait_queue() {
    let _lock = SERIAL.lock();

    const NUM_TASKS: usize = 10;

    static WQ1: WaitQueue = WaitQueue::new();
    static WQ2: WaitQueue = WaitQueue::new();
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    static HAS_ERROR: AtomicBool = AtomicBool::new(false);
    COUNTER.store(0, Ordering::Release);
    HAS_ERROR.store(false, Ordering::Release);

    for _ in 0..NUM_TASKS {
        axtask::spawn(move || {
            COUNTER.fetch_add(1, Ordering::Release);
            WQ1.notify_one(true);
            WQ2.wait();

            if current().in_wait_queue() {
                HAS_ERROR.store(true, Ordering::Release);
            }

            COUNTER.fetch_sub(1, Ordering::Release);
            WQ1.notify_one(true);
        });
    }

    WQ1.wait_until(|| COUNTER.load(Ordering::Acquire) == NUM_TASKS);
    ax_assert_eq!(COUNTER.load(Ordering::Acquire), NUM_TASKS);
    ax_assert!(!current().in_wait_queue());
    WQ2.notify_all(true);

    WQ1.wait_until(|| COUNTER.load(Ordering::Acquire) == 0);
    ax_assert_eq!(COUNTER.load(Ordering::Acquire), 0);
    ax_assert!(!current().in_wait_queue());
    ax_assert!(!HAS_ERROR.load(Ordering::Acquire));
}

#[def_test]
fn test_task_join() {
    let _lock = SERIAL.lock();

    const NUM_TASKS: usize = 10;
    let mut tasks = Vec::with_capacity(NUM_TASKS);

    for i in 0..NUM_TASKS {
        tasks.push(axtask::spawn_raw(
            move || {
                axtask::yield_now();
                axtask::exit(i as _);
            },
            format!("T{}", i),
            0x1000,
        ));
    }

    for (i, task) in tasks.iter().enumerate() {
        ax_assert_eq!(task.join(), Some(i as _));
    }
}
