//! The embassy single-thread executor.
#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use axasync::cell::StaticCell;
use axasync::executor::Executor;
use axasync::time::Timer;
use core::hint::black_box;

fn busy_work(nano: u64) -> u64 {
    let mut total = 0;
    for _ in 0..nano {
        let mut x = 0;
        for _ in 0..nano {
            x += 1;
        }
        total = black_box(total + x);
    }
    total
}

#[axasync::executor::task(pool_size = 4)]
async fn tick(_sec: u64, busy_nano: u64) {
    for i in 0..10 {
        println!("embassy tick {}: {}/s, {}", _sec, _sec * i, i);
        busy_work(busy_nano);
        Timer::after_secs(_sec).await;
    }
    panic!("tick finished");
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Embassy Test");
    let exec = EXECUTOR.init(Executor::new());
    exec.run(|sp| {
        for i in 1..4 {
            sp.spawn(tick(i, 0)).unwrap();
        }
    })
}
