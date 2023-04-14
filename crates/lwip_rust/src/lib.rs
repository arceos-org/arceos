#![no_std]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
pub mod bindings;

pub fn lwip_rust_init() {
    unsafe {
        bindings::lwip_init();
    }
}
