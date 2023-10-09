#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::thread;

use std::time::Instant;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let mut sum: u128 = 0;

    for i in 0..100 {
        let start = Instant::now();
        for _ in 0..50 {
            let handle = thread::spawn(|| {});
            handle.join().unwrap();
        }
        let duration = start.elapsed();
        sum = sum + duration.as_nanos();
        println!("i: {} duration: {}", i, duration.as_nanos());
    }

    // 以纳秒单位输出消耗的时间
    println!("Time: {}", sum / 100);
}
