#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;
use std::thread;

use std::time::Instant;
extern crate alloc;
use alloc::vec;
#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let test_list = [1, 2, 5, 10, 15, 20, 25, 50, 60, 70, 80, 90, 100];
    for test_num in test_list {
        let mut sum: u128 = 0;

        for _ in 0..10 {
            let mut handles = vec![];
            let start = Instant::now();
            for _ in 0..50 {
                let f = move |num: i32| {
                    for _ in 0..num {
                        thread::yield_now();
                    }
                };
                let handle = thread::spawn(move || f(test_num));
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            let duration = start.elapsed();
            sum += duration.as_nanos();
        }

        // 以纳秒单位输出消耗的时间
        println!("Yield: Num: {}, Time: {}", test_num, sum / 10);
    }
}
