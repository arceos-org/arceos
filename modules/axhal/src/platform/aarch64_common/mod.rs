mod boot;

pub mod generic_timer;
#[cfg(not(platform_family = "aarch64-raspi"))]
pub mod psci;

#[cfg(feature = "irq")]
pub mod gic;

#[cfg(not(any(
    platform_family = "aarch64-bsta1000b",
    platform_family = "aarch64-rk3588j"
)))]
pub mod pl011;
