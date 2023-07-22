#![no_std]
#![no_main]

extern crate axstd;

#[no_mangle]
fn main() {
    axstd::println!("Benchmarking bandwidth...");
    axnet::bench_transmit();
    // axnet::bench_receive();
}
