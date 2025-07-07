extern crate alloc;

use alloc::format;
use axdriver_base::DeviceType;
use axdriver_block::BlockDriverOps;
use axdriver_virtio::MmioTransport;
use axhal::mem::PhysAddr;
use rdrive::{
    DriverGeneric, PlatformDevice, driver::block::*, module_driver, probe::OnProbeError,
    register::FdtInfo,
};

use crate::dyn_drivers::blk::maping_dev_err_to_io_err;
use crate::dyn_drivers::iomap;
use crate::virtio::VirtIoHalImpl;

type Device<T> = axdriver_virtio::VirtIoBlkDev<VirtIoHalImpl, T>;

module_driver!(
    name: "Virtio Block",
    level: ProbeLevel::PostKernel,
    priority: ProbePriority::DEFAULT,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["virtio,mmio"],
            on_probe: probe
        }
    ],
);

fn probe(info: FdtInfo<'_>, plat_dev: PlatformDevice) -> Result<(), OnProbeError> {
    let base_reg = info
        .node
        .reg()
        .and_then(|mut regs| regs.next())
        .ok_or(OnProbeError::other(alloc::format!(
            "[{}] has no reg",
            info.node.name()
        )))?;

    let mmio_size = base_reg.size.unwrap_or(0x1000);
    let mmio_base = PhysAddr::from_usize(base_reg.address as usize);

    let mmio_base = iomap(mmio_base, mmio_size)?.as_ptr();

    let (ty, transport) =
        axdriver_virtio::probe_mmio_device(mmio_base, mmio_size).ok_or(OnProbeError::NotMatch)?;

    if ty != DeviceType::Block {
        return Err(OnProbeError::NotMatch);
    }

    let dev = Device::try_new(transport).map_err(|e| {
        OnProbeError::other(format!(
            "failed to initialize Virtio Block device at [PA:{mmio_base:?},): {e:?}"
        ))
    })?;

    let dev = BlockDivce(dev);
    plat_dev.register_block(dev);
    debug!("virtio block device registered successfully");
    Ok(())
}

struct BlockDivce(Device<MmioTransport>);

impl DriverGeneric for BlockDivce {
    fn open(&mut self) -> Result<(), rdrive::KError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), rdrive::KError> {
        Ok(())
    }
}

impl rdrive::driver::block::Interface for BlockDivce {
    fn num_blocks(&self) -> usize {
        self.0.num_blocks() as _
    }

    fn block_size(&self) -> usize {
        self.0.block_size()
    }

    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<(), io::Error> {
        self.0
            .read_block(block_id as u64, buf)
            .map_err(maping_dev_err_to_io_err)
    }
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<(), io::Error> {
        self.0
            .write_block(block_id as u64, buf)
            .map_err(maping_dev_err_to_io_err)
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        self.0.flush().map_err(maping_dev_err_to_io_err)
    }
}
