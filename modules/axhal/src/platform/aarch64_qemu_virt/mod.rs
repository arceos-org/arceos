#[cfg(feature = "smp")]
pub mod mp;

pub mod misc {
    pub use crate::platform::aarch64_common::psci::system_off as terminate;
}
