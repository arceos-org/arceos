#![no_std]
#![allow(unused)]
#![allow(missing_docs)]

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(target_arch = "loongarch64")] {
        pub mod asm;
        pub mod consts;
        pub mod cpu;
        pub mod extioi;
        pub mod ipi;
        pub mod loongson;
        pub mod ls7a;
        pub mod mem;
        pub mod register;
        pub mod tlb;

        pub const VALEN: usize = 48;
        pub const PALEN: usize = 48;
    }
}
