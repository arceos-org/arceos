#![no_std]
#![no_main]

#[macro_use]
extern crate axstd;

use axstd::thread;
use axstd::time::{Duration, Instant};
use core::sync::atomic::{AtomicUsize, Ordering};

const NUM_TASKS: usize = 5;

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    println!("Hello, main task!");
    let now = Instant::now();
    thread::sleep(Duration::from_secs(1));
    let elapsed = now.elapsed();
    println!("main task sleep for {:?}", elapsed);

    // backgroud ticks, 0.5s x 30 = 15s
    thread::spawn(|| {
        for i in 0..30 {
            info!("  tick {}", i);
            thread::sleep(Duration::from_millis(500));
        }
    });

    // task n: sleep 3 x n (sec)
    for i in 0..NUM_TASKS {
        thread::spawn(move || {
            let sec = i + 1;
            for j in 0..3 {
                println!("task {} sleep {} seconds ({}) ...", i, sec, j);
                let now = Instant::now();
                thread::sleep(Duration::from_secs(sec as _));
                let elapsed = now.elapsed();
                info!("task {} actual sleep {:?} seconds ({}).", i, elapsed, j);
            }
            FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
        });
    }

    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        thread::sleep(Duration::from_millis(10));
    }
    println!("Sleep tests run OK!");
}
