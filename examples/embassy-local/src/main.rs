#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use axasync::spawner;
use std::time::Duration;
use std::{
    boxed::Box,
    thread::{self, sleep},
};

fn busy_work() {
    for _ in 0..1000 {
        let mut x = 0;
        for _ in 0..1000 {
            x += 1;
        }
    }
}

fn tick_raw(_sec: u64, busy_nano: u64) -> embassy_executor::SpawnToken<impl Sized> {
    let task = Box::leak(Box::new(embassy_executor::raw::TaskStorage::new()));
    task.spawn(move || async move {
        for i in 0..10 {
            println!("embassy tick: {}/s, {} times", _sec * i, i);
            busy_work();
            embassy_time::Timer::after_secs(_sec).await;
        }
        panic!("tick finished");
    })
}

#[embassy_executor::task]
async fn tick(_sec: u64, busy_nano: u64) {
    for i in 0..10 {
        println!("embassy tick: {}/s, {} times", _sec * i, i);
        busy_work();
        embassy_time::Timer::after_secs(_sec).await;
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
    // Avoid
    sleep(Duration::from_millis(1000 * 15 as u64));
}
