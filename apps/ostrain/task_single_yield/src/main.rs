#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;
use std::thread;

use std::time::Instant;
#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let test_list = [1, 2, 5, 10, 15, 20, 25, 50, 60, 70, 80, 90, 100];
    for test_num in test_list {
        let mut sum: u128 = 0;

        for _ in 0..10 {
            let start = Instant::now();
            for _ in 0..test_num * 100 {
                thread::yield_now();
            }
            let duration = start.elapsed();
            sum = sum + duration.as_nanos();
        }

        // 以纳秒单位输出消耗的时间
        println!("Yield: Num: {}, Time: {}", test_num, sum / 10);
    }
}
