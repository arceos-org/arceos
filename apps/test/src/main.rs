#![no_std]
#![no_main]

use core::time::Duration;



#[macro_use]
extern crate libax;
extern crate alloc;

mod test_mem;
mod test_sleep;

#[no_mangle]
fn main() {
    libax::println!("Hello, testcases!");
    //test_sleep::main();
    //test_mem::main();
    sync_test();
}

#[allow(unused)]
fn sync_test() {
    use libax::Mutex;
    use alloc::sync::Arc;
    let counter = Arc::new(Mutex::new(0));

    for i in 0..10 {
        let counter_inner = counter.clone();
        libax::task::spawn(move || {
            for j in 0..10 {
                let mut counter_locked = counter_inner.lock();
                println!("Task {}-{} Locked!", i, j);
                *counter_locked += i * 10 + j;
                drop(counter_locked);
                libax::task::yield_now();
            }
        })
    }
    libax::task::sleep(Duration::from_secs(1));
    println!("{:?}", counter);
}
