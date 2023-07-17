#![no_std]
#![no_main]

extern crate libax;

#[no_mangle]
fn main() {
    libax::println!("Benchmarking bandwidth...");
    axnet::bench_transmit();
    // axnet::bench_receive();
}
