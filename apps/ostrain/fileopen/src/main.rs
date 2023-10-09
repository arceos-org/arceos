#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::fs::File;
use std::time::Instant;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let path = "test.txt";
    let mut sum: u128 = 0;
    for _ in 0..10 {
        let start = Instant::now();
        for _ in 0..50 {
            let f = File::open(path).unwrap();
            drop(f);
            // 不需要关闭文件
        }
        let duration = start.elapsed();
        sum = sum + duration.as_nanos();
    }

    // 以纳秒单位输出消耗的时间
    println!("Time: {}", sum / 10);
}
