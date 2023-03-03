#![no_std]

#[macro_use]
extern crate log;

#[cfg(feature = "virtio")]
mod virtio;

use tuple_for_each::TupleForEach;

#[cfg(feature = "virtio-blk")]
pub use self::virtio::VirtIoBlockDev;
#[cfg(feature = "virtio-net")]
pub use self::virtio::VirtIoNetDev;

#[derive(TupleForEach)]
pub struct BlockDevices(
    #[cfg(feature = "virtio-blk")] pub VirtIoBlockDev,
    // e.g. #[cfg(feature = "nvme")] pub nvme::NVMeDev,
);

#[derive(TupleForEach)]
pub struct NetDevices(
    #[cfg(feature = "virtio-net")] pub VirtIoNetDev,
    // e.g. #[cfg(feature = "e1000")] pub e1000::E1000Dev,
);

pub struct AllDevices {
    pub block: BlockDevices,
    pub net: NetDevices,
}

impl AllDevices {
    fn probe() -> Self {
        Self {
            block: BlockDevices(
                #[cfg(feature = "virtio-blk")]
                Self::probe_virtio_blk().expect("no virtio-blk device found"),
            ),
            net: NetDevices(
                #[cfg(feature = "virtio-net")]
                Self::probe_virtio_net().expect("no virtio-net device found"),
            ),
        }
    }
}

pub fn init_drivers() -> AllDevices {
    info!("Initialize device drivers...");

    AllDevices::probe()
}
