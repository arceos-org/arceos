#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(const_trait_impl)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(doc_auto_cfg)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

mod platform;

pub mod arch;
pub mod cpu;
pub mod irq;
pub mod mem;
pub mod time;
pub mod trap;

#[cfg(feature = "paging")]
pub mod paging;

pub mod console {
    pub use super::platform::console::*;

    pub fn write_bytes(bytes: &[u8]) {
        for c in bytes {
            putchar(*c);
        }
    }
}

pub mod misc {
    pub use super::platform::misc::*;
}

#[cfg(feature = "smp")]
pub mod mp {
    pub use super::platform::mp::*;
}
