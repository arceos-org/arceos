//! TODO: generate registered drivers in `for_each_drivers!` automatically.

#![allow(unused_macros)]

macro_rules! register_net_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type AxNetDevice = $device_type;
    };
}

macro_rules! register_block_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type AxBlockDevice = $device_type;
    };
}

macro_rules! register_display_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type AxDisplayDevice = $device_type;
    };
}

macro_rules! for_each_drivers {
    (type $drv_type:ident, $code:block) => {{
        #[allow(unused_imports)]
        use crate::drivers::DriverProbe;
        #[cfg(feature = "virtio")]
        #[allow(unused_imports)]
        use crate::virtio::{self, VirtIoDevMeta};

        #[cfg(net_dev = "virtio-net")]
        {
            type $drv_type = <virtio::VirtIoNet as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(block_dev = "virtio-blk")]
        {
            type $drv_type = <virtio::VirtIoBlk as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(display_dev = "virtio-gpu")]
        {
            type $drv_type = <virtio::VirtIoGpu as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(block_dev = "ramdisk")]
        {
            type $drv_type = crate::drivers::RamDiskDriver;
            $code
        }
        #[cfg(block_dev = "bcm2835-sdhci")]
        {
            type $drv_type = crate::drivers::BcmSdhciDriver;
            $code
        }
        #[cfg(net_dev = "ixgbe")]
        {
            type $drv_type = crate::drivers::IxgbeDriver;
            $code
        }
    }};
}
