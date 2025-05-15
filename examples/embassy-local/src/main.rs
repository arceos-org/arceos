#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use embassy_futures::yield_now;
use std::Executor;
use std::time::Duration;
use std::{
    boxed::Box,
    thread::{self, sleep},
};

fn tick(_sec: u64, busy_nano: u64) -> embassy_executor::SpawnToken<impl Sized> {
    let task = Box::leak(Box::new(embassy_executor::raw::TaskStorage::new()));
    task.spawn(move || async move {
        for i in 0..10 {
            println!("embassy tick {}/s :{}", _sec, i);
            for _ in 0..busy_nano {
                // Simulate some busy work
                let _ = (0..1000).fold(0, |acc, x| acc + x);
            }
            embassy_time::Timer::after_secs(_sec).await;
        }
        panic!("tick finished");
    })
}

fn idle() -> embassy_executor::SpawnToken<impl Sized> {
    let task = Box::leak(Box::new(embassy_executor::raw::TaskStorage::new()));
    task.spawn(move || async move {
        loop {
            yield_now().await;
        }
    })
}

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Embassy Test");
    for i in 1..4 {
        println!("spawned thread {}", i);
        thread::spawn(move || {
            for j in 0..10 {
                println!("spawn tick {}: {}", i, j);
                sleep(Duration::from_millis(500 * i as u64));
            }
            println!("spawned thread {} finished", i);
        });
    }
    let exec = Box::leak(Box::new(Executor::new()));
    exec.run(|s| {
        // s.spawn(idle()).unwrap();
        s.spawn(tick(1, 0)).unwrap();
    });
}
