mod boot;

#[cfg(not(platform_family = "aarch64-raspi"))]
pub mod psci;

#[cfg(feature = "irq")]
mod gic;

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::gic::*;
}

mod generic_timer;
pub mod time {
    pub use super::generic_timer::*;
}

#[cfg(platform_family = "aarch64-bsta1000b")]
mod dw_apb_uart;

#[cfg(any(platform_family = "aarch64-raspi", platform_family = "aarch64-qemu-virt"))]
mod pl011;

pub mod console {
    #[cfg(platform_family = "aarch64-bsta1000b")]
    pub use super::dw_apb_uart::*;

    #[cfg(any(platform_family = "aarch64-raspi", platform_family = "aarch64-qemu-virt"))]
    pub use super::pl011::*;
}
