mod boot;

pub mod generic_timer;
pub mod pl011;
pub mod psci;

#[cfg(feature = "irq")]
pub mod gic;
