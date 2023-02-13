#![no_std]

#[cfg(feature = "virtio")]
mod virtio;

use lazy_init::LazyInit;

static DEVICES: LazyInit<AllDevices> = LazyInit::new();

pub struct BlockDevices(
    #[cfg(feature = "virtio-blk")] pub self::virtio::VirtIoBlockDev,
    // #[cfg(feature = "nvme")] pub nvme::NVMeDev,
);

pub struct NetDevices(
    #[cfg(feature = "virtio-net")] pub self::virtio::VirtIoNetDev,
    // #[cfg(feature = "e1000")] pub e1000::E1000Dev,
);

pub struct AllDevices {
    block: BlockDevices,
    net: NetDevices,
}

impl AllDevices {
    pub fn probe() -> Self {
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

pub fn block_devices() -> &'static BlockDevices {
    &DEVICES.block
}

pub fn net_devices() -> &'static NetDevices {
    &DEVICES.net
}

pub fn init_drivers() {
    let all_devices = AllDevices::probe();
    DEVICES.init_by(all_devices);
}
