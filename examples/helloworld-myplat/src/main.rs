#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "aarch64", feature = "aarch64-qemu-virt"))] {
        extern crate axplat_aarch64_qemu_virt;
    } else if #[cfg(all(target_arch = "aarch64", feature = "aarch64-raspi4"))] {
        extern crate axplat_aarch64_raspi;
    } else if #[cfg(all(target_arch = "aarch64", feature = "aarch64-phytium-pi"))] {
        extern crate axplat_aarch64_phytium_pi;
    } else if #[cfg(all(target_arch = "aarch64", feature = "aarch64-bsta1000b"))] {
        extern crate axplat_aarch64_bsta1000b;
    } else if #[cfg(all(target_arch = "x86_64", feature = "x86-pc"))] {
        extern crate axplat_x86_pc;
    } else if #[cfg(all(target_arch = "riscv64", feature = "riscv64-qemu-virt"))] {
        extern crate axplat_riscv64_qemu_virt;
    } else if #[cfg(all(target_arch = "loongarch64", feature = "loongarch64-qemu-virt"))] {
        extern crate axplat_loongarch64_qemu_virt;
    } else {
        #[cfg(target_os = "none")] // ignore in rust-analyzer & cargo test
        compile_error!("No platform crate linked!\n\nPlease add `extern crate <platform>` in your code.");
    }
}

#[cfg(feature = "axstd")]
use axstd::println;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    println!("Hello, world!");
}
