use axtask::{unpark_task, TaskId};

use core::{sync::atomic::AtomicU64, task::Waker};

use alloc::collections::BTreeMap;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use kspin::SpinNoIrq;
use log::info;
use static_cell::StaticCell;

use crate::executor::{self};

static EXECUTOR: StaticCell<executor::Executor> = StaticCell::new();

pub fn runtime<F>(initial: F) -> !
where
    F: FnOnce(Spawner) + Send + 'static,
{
    let exec = EXECUTOR.init(executor::Executor::new());
    exec.run(initial)
}


#[embassy_executor::task]
async fn tick(_sec: u64) {
    for _ in 0..4 {
        info!("tick for {} sec", _sec);
        yield_now().await;
    }
}

#[embassy_executor::task]
async fn tick_2(_sec: u64) {
    for _ in 0..4 {
        info!("tick for {} sec", _sec);
        yield_now().await;
    }
}

pub fn init() {
    let exec = EXECUTOR.init(executor::Executor::new());
    
    exec.run(|s| {
        s.spawn(tick(1)).unwrap();
        s.spawn(tick_2(2)).unwrap();
    })
}
