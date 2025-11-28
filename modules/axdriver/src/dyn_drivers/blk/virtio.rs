extern crate alloc;

use alloc::format;
use axdriver_base::DeviceType;
use axdriver_block::BlockDriverOps;
use axdriver_virtio::MmioTransport;
use axhal::mem::PhysAddr;
use rdrive::{
    DriverGeneric, PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo,
};

use super::PlatformDeviceBlock;
use crate::dyn_drivers::blk::maping_dev_err_to_blk_err;
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

    let dev = BlockDivce { dev: Some(dev) };
    plat_dev.register_block(dev);
    debug!("virtio block device registered successfully");
    Ok(())
}

struct BlockDivce {
    dev: Option<Device<MmioTransport>>,
}

struct BlockQueue {
    raw: Device<MmioTransport>,
}

impl DriverGeneric for BlockDivce {
    fn open(&mut self) -> Result<(), rdrive::KError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), rdrive::KError> {
        Ok(())
    }
}

impl rdif_block::Interface for BlockDivce {
    fn create_queue(&mut self) -> Option<alloc::boxed::Box<dyn rdif_block::IQueue>> {
        self.dev
            .take()
            .map(|dev| alloc::boxed::Box::new(BlockQueue { raw: dev }) as _)
    }

    fn enable_irq(&mut self) {
        todo!()
    }

    fn disable_irq(&mut self) {
        todo!()
    }

    fn is_irq_enabled(&self) -> bool {
        false
    }

    fn handle_irq(&mut self) -> rdif_block::Event {
        rdif_block::Event::none()
    }
}

impl rdif_block::IQueue for BlockQueue {
    fn num_blocks(&self) -> usize {
        self.raw.num_blocks() as _
    }

    fn block_size(&self) -> usize {
        self.raw.block_size()
    }

    fn id(&self) -> usize {
        0
    }

    fn buff_config(&self) -> rdif_block::BuffConfig {
        rdif_block::BuffConfig {
            dma_mask: u64::MAX,
            align: 0x1000,
            size: self.block_size(),
        }
    }

    fn submit_request(
        &mut self,
        request: rdif_block::Request<'_>,
    ) -> Result<rdif_block::RequestId, rdif_block::BlkError> {
        let id = request.block_id;
        match request.kind {
            rdif_block::RequestKind::Read(mut buffer) => {
                self.raw
                    .read_block(id as _, &mut buffer)
                    .map_err(maping_dev_err_to_blk_err)?;
                Ok(rdif_block::RequestId::new(0))
            }
            rdif_block::RequestKind::Write(items) => {
                self.raw
                    .write_block(id as _, items)
                    .map_err(maping_dev_err_to_blk_err)?;
                Ok(rdif_block::RequestId::new(0))
            }
        }
    }

    fn poll_request(
        &mut self,
        _request: rdif_block::RequestId,
    ) -> Result<(), rdif_block::BlkError> {
        Ok(())
    }
}
