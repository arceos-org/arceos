#![no_std]

mod fs;

#[macro_use]
extern crate libax;

extern crate alloc;

pub fn init() {
    fs::init_fs();
    info!("FS inited");
    fs::run();
}
