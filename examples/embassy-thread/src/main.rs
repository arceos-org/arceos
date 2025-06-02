#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use axasync::executor::spawner;
use axasync::time::Timer;
use core::hint::black_box;
use std::time::Duration;
use std::thread::{self, sleep};

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

#[embassy_executor::task]
async fn tick(_sec: u64, busy_nano: u64) {
    for i in 0..10 {
        println!("embassy tick: {}/s, {} times", _sec * i, i);
        busy_work(busy_nano);
        Timer::after_secs(_sec).await;
    }
    panic!("tick finished");
}

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Embassy Test");
    for i in 1..4 {
        println!("spawned thread {}", i);
        thread::spawn(move || {
            for j in 0..5 {
                println!("thread {} tick: {}/s {} times", i, j * i, j);
                sleep(Duration::from_millis(1000 * i as u64));
            }
            println!("thread {} finished", i);
        });
    }

    spawner().spawn(tick(1, 0)).unwrap();
    // Avoid shut down immediately
    sleep(Duration::from_millis(1000 * 15 as u64));
}
