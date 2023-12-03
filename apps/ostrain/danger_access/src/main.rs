#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::thread;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let handler = thread::spawn(|| {
        let pt: *const u8 = 0x1 as *const u8;
        unsafe {
            let val = *pt;
            println!("val: {}", val);
        }
    });
    let _ = handler.join();
    println!("Solve page fault success!");
}
