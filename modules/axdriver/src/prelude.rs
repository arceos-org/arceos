//! Device driver prelude that includes some traits and types.

pub use axdriver_base::{BaseDriverOps, DevError, DevResult, DeviceType};

#[cfg(feature = "block")]
pub use {crate::structs::AxBlockDevice, axdriver_block::BlockDriverOps};
#[cfg(feature = "display")]
pub use {crate::structs::AxDisplayDevice, axdriver_display::DisplayDriverOps};
#[cfg(feature = "net")]
pub use {crate::structs::AxNetDevice, axdriver_net::NetDriverOps};
