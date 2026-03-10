use alloc::vec::Vec;

pub fn probe_all_devices() -> Vec<super::AxDeviceEnum> {
    if let Err(err) = axplat_dyn::drivers::probe_all_devices() {
        error!("failed to probe dynamic platform devices: {err:?}");
        return Vec::new();
    }

    #[allow(unused_mut)]
    let mut devices = Vec::new();

    #[cfg(feature = "block")]
    for dev in axplat_dyn::drivers::take_block_devices() {
        devices.push(super::AxDeviceEnum::Block(dev));
    }

    devices
}
