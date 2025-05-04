#![feature(impl_trait_in_assoc_type)]
#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use embassy_futures::yield_now;
use std::boxed::Box;

mod executor;

fn tick(sec: u64, f: fn()) -> embassy_executor::SpawnToken<impl Sized> {
    let task = Box::leak(Box::new(embassy_executor::raw::TaskStorage::new()));
    task.spawn(move || async move {
        for _ in 0..10 {
            f();
            yield_now().await;
        }
    })
}

// used as a tick timer to delay
fn idle() -> embassy_executor::SpawnToken<impl Sized> {
    let task = Box::leak(Box::new(embassy_executor::raw::TaskStorage::new()));
    task.spawn(move || async move {
        for _ in 0..10 {
            yield_now().await;
        }
    })
}

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Embassy Test");
    let exec = Box::leak(Box::new(executor::Executor::new()));
    exec.run(|s| {
        s.spawn(idle()).unwrap();
        s.spawn(tick(1, || {println!("tick for 1 sec")})).unwrap();
        s.spawn(tick(2, || {println!("tick for 2 sec")})).unwrap();
    });
}
