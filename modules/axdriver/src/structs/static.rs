#[cfg(feature = "block")]
pub use crate::drivers::AxBlockDevice;
#[cfg(feature = "display")]
pub use crate::drivers::AxDisplayDevice;
#[cfg(feature = "input")]
pub use crate::drivers::AxInputDevice;
#[cfg(feature = "net")]
pub use crate::drivers::AxNetDevice;
#[cfg(feature = "vsock")]
pub use crate::drivers::AxVsockDevice;

impl super::AxDeviceEnum {
    /// Constructs a network device.
    #[cfg(feature = "net")]
    pub const fn from_net(dev: AxNetDevice) -> Self {
        Self::Net(dev)
    }

    /// Constructs a block device.
    #[cfg(feature = "block")]
    pub const fn from_block(dev: AxBlockDevice) -> Self {
        Self::Block(dev)
    }

    /// Constructs a display device.
    #[cfg(feature = "display")]
    pub const fn from_display(dev: AxDisplayDevice) -> Self {
        Self::Display(dev)
    }

    /// Constructs a display device.
    #[cfg(feature = "input")]
    pub const fn from_input(dev: AxInputDevice) -> Self {
        Self::Input(dev)
    }

    /// Constructs a socket device.
    #[cfg(feature = "vsock")]
    pub const fn from_vsock(dev: AxVsockDevice) -> Self {
        Self::Vsock(dev)
    }
}
