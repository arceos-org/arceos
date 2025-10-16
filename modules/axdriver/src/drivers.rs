//! Defines types and probe methods of all supported devices.

#![allow(unused_imports, dead_code)]

use axdriver_base::DeviceType;
#[cfg(feature = "block")]
use axdriver_block::BlockDriverOps;
#[cfg(feature = "net")]
use axdriver_net::BaseDriverOps;
#[cfg(feature = "bus-pci")]
use axdriver_pci::{DeviceFunction, DeviceFunctionInfo, PciRoot};

pub use super::dummy::*;
use crate::AxDeviceEnum;
#[cfg(feature = "virtio")]
use crate::virtio::{self, VirtIoDevMeta};

pub trait DriverProbe {
    fn probe_global() -> Option<AxDeviceEnum> {
        None
    }

    #[cfg(bus = "mmio")]
    fn probe_mmio(_mmio_base: usize, _mmio_size: usize) -> Option<AxDeviceEnum> {
        None
    }

    #[cfg(bus = "pci")]
    fn probe_pci(
        _root: &mut PciRoot,
        _bdf: DeviceFunction,
        _dev_info: &DeviceFunctionInfo,
    ) -> Option<AxDeviceEnum> {
        None
    }
}

#[cfg(net_dev = "virtio-net")]
register_net_driver!(
    <virtio::VirtIoNet as VirtIoDevMeta>::Driver,
    <virtio::VirtIoNet as VirtIoDevMeta>::Device
);

#[cfg(block_dev = "virtio-blk")]
register_block_driver!(
    <virtio::VirtIoBlk as VirtIoDevMeta>::Driver,
    <virtio::VirtIoBlk as VirtIoDevMeta>::Device
);

#[cfg(display_dev = "virtio-gpu")]
register_display_driver!(
    <virtio::VirtIoGpu as VirtIoDevMeta>::Driver,
    <virtio::VirtIoGpu as VirtIoDevMeta>::Device
);

#[cfg(input_dev = "virtio-input")]
register_input_driver!(
    <virtio::VirtIoInput as VirtIoDevMeta>::Driver,
    <virtio::VirtIoInput as VirtIoDevMeta>::Device
);

#[cfg(vsock_dev = "virtio-socket")]
register_vsock_driver!(
    <virtio::VirtIoSocket as VirtIoDevMeta>::Driver,
    <virtio::VirtIoSocket as VirtIoDevMeta>::Device
);

cfg_if::cfg_if! {
    if #[cfg(block_dev = "ramdisk")] {
        use axdriver_block::ramdisk::RamDisk;
        use axhal::mem::phys_to_virt;

        pub struct RamDiskDriver;
        register_block_driver!(RamDiskDriver, RamDisk);

        impl DriverProbe for RamDiskDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                // FIXME: this configuration is specific to 2k1000la!
                let (start, size) = axconfig::devices::INITRD_RANGE;
                let initrd = unsafe { RamDisk::new(phys_to_virt(start.into()).into(), size) };
                Some(AxDeviceEnum::from_block(initrd))
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(block_dev = "sdmmc-gpt")] {
        use axdriver_block::{gpt::GptPartitionDev, sdmmc::SdMmcDriver};
        use axhal::mem::phys_to_virt;

        pub struct SdMmcGptDriver;
        register_block_driver!(SdMmcGptDriver, GptPartitionDev<SdMmcDriver>);

        impl DriverProbe for SdMmcGptDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                // FIXME: this configuration is specific to vf2!
                let root = "root".parse().unwrap();
                let sdmmc = unsafe { SdMmcDriver::new(phys_to_virt(0x1602_0000.into()).into()) };
                GptPartitionDev::try_new(sdmmc, |_, part| part.name == root)
                    .ok()
                    .map(AxDeviceEnum::from_block)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(block_dev = "ahci-gpt")] {
        use axdriver_base::{DevResult, DevError};
        use axdriver_block::gpt::GptPartitionDev;
        use axhal::mem::phys_to_virt;

        pub struct AhciDriver(ahci_driver::ahci_device);

        unsafe impl Send for AhciDriver {}
        unsafe impl Sync for AhciDriver {}

        impl BaseDriverOps for AhciDriver {
            fn device_name(&self) -> &str {
                "ahci"
            }

            fn device_type(&self) -> DeviceType {
                DeviceType::Block
            }
        }

        impl BlockDriverOps for AhciDriver {
            fn block_size(&self) -> usize {
                512
            }

            fn num_blocks(&self) -> u64 {
                // FIXME: this configuration is not reusable!
                250069680
            }

            fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
                let res = ahci_driver::ahci_sata_read_common(&self.0, block_id, 1, buf.as_mut_ptr());
                if res == 0 {
                    Err(DevError::Io)
                } else {
                    Ok(())
                }
            }

            fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
                let res = ahci_driver::ahci_sata_write_common(&self.0, block_id, 1, buf.as_ptr().cast_mut());
                if res == 0 {
                    Err(DevError::Io)
                } else {
                    Ok(())
                }
            }

            fn flush(&mut self) -> DevResult {
                Ok(())
            }
        }

        pub struct AhciGptDriver;
        register_block_driver!(AhciGptDriver, GptPartitionDev<AhciDriver>);

        impl DriverProbe for AhciGptDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                let mut dev = ahci_driver::ahci_device::default();
                ahci_driver::ahci_init(&mut dev);
                let dev = AhciDriver(dev);
                GptPartitionDev::try_new(dev, |_, _| true)
                    .ok()
                    .map(AxDeviceEnum::from_block)
            }
        }

        #[unsafe(no_mangle)]
        unsafe extern "C" fn ahci_printf(fmt: *const u8, mut args: ...) -> i32 {
            use printf_compat::{format, output};

            let mut s = alloc::string::String::new();
            let bytes_written = unsafe { format(fmt as _, args.as_va_list(), output::fmt_write(&mut s)) };
            trace!("ahci_driver: {}", s.trim());

            bytes_written
        }

        #[unsafe(no_mangle)]
        extern "C" fn ahci_phys_to_uncached(pa: u64) -> u64 {
            axhal::mem::phys_to_virt((pa as usize).into()).as_usize() as _
        }

        #[unsafe(no_mangle)]
        extern "C" fn ahci_virt_to_phys(va: u64) -> u64 {
            axhal::mem::virt_to_phys((va as usize).into()).as_usize() as _
        }

        #[unsafe(no_mangle)]
        extern "C" fn ahci_mdelay(ms: u64) {
            let start = axhal::time::wall_time_nanos();
            while axhal::time::wall_time_nanos() - start < ms * 1_000_000 {
                core::hint::spin_loop();
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(block_dev = "bcm2835-sdhci")]{
        pub struct BcmSdhciDriver;
        register_block_driver!(BcmSdhciDriver, axdriver_block::bcm2835sdhci::SDHCIDriver);

        impl DriverProbe for BcmSdhciDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                debug!("mmc probe");
                axdriver_block::bcm2835sdhci::SDHCIDriver::try_new().ok().map(AxDeviceEnum::from_block)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(net_dev = "ixgbe")] {
        use crate::ixgbe::IxgbeHalImpl;
        use axhal::mem::phys_to_virt;
        pub struct IxgbeDriver;
        register_net_driver!(IxgbeDriver, axdriver_net::ixgbe::IxgbeNic<IxgbeHalImpl, 1024, 1>);
        impl DriverProbe for IxgbeDriver {
            #[cfg(bus = "pci")]
            fn probe_pci(
                    root: &mut axdriver_pci::PciRoot,
                    bdf: axdriver_pci::DeviceFunction,
                    dev_info: &axdriver_pci::DeviceFunctionInfo,
                ) -> Option<crate::AxDeviceEnum> {
                    use axdriver_net::ixgbe::{INTEL_82599, INTEL_VEND, IxgbeNic};
                    if dev_info.vendor_id == INTEL_VEND && dev_info.device_id == INTEL_82599 {
                        // Intel 10Gb Network
                        info!("ixgbe PCI device found at {:?}", bdf);

                        // Initialize the device
                        // These can be changed according to the requirments specified in the ixgbe init function.
                        const QN: u16 = 1;
                        const QS: usize = 1024;
                        let bar_info = root.bar_info(bdf, 0).unwrap();
                        match bar_info {
                            axdriver_pci::BarInfo::Memory {
                                address,
                                size,
                                ..
                            } => {
                                let ixgbe_nic = IxgbeNic::<IxgbeHalImpl, QS, QN>::init(
                                    phys_to_virt((address as usize).into()).into(),
                                    size as usize
                                )
                                .expect("failed to initialize ixgbe device");
                                return Some(AxDeviceEnum::from_net(ixgbe_nic));
                            }
                            axdriver_pci::BarInfo::IO { .. } => {
                                error!("ixgbe: BAR0 is of I/O type");
                                return None;
                            }
                        }
                    }
                    None
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(net_dev = "fxmac")]{
        use axalloc::{UsageKind, global_allocator};
        use axhal::mem::PAGE_SIZE_4K;

        #[crate_interface::impl_interface]
        impl axdriver_net::fxmac::KernelFunc for FXmacDriver {
            fn virt_to_phys(addr: usize) -> usize {
                axhal::mem::virt_to_phys(addr.into()).into()
            }

            fn phys_to_virt(addr: usize) -> usize {
                axhal::mem::phys_to_virt(addr.into()).into()
            }

            fn dma_alloc_coherent(pages: usize) -> (usize, usize) {
                let Ok(vaddr) = global_allocator().alloc_pages(pages, PAGE_SIZE_4K, UsageKind::Dma) else {
                    error!("failed to alloc pages");
                    return (0, 0);
                };
                let paddr = axhal::mem::virt_to_phys((vaddr).into());
                debug!("alloc pages @ vaddr={:#x}, paddr={:#x}", vaddr, paddr);
                (vaddr, paddr.as_usize())
            }

            fn dma_free_coherent(vaddr: usize, pages: usize) {
                global_allocator().dealloc_pages(vaddr, pages, UsageKind::Dma);
            }

            fn dma_request_irq(_irq: usize, _handler: fn()) {
                warn!("unimplemented dma_request_irq for fxmax");
            }
        }

        register_net_driver!(FXmacDriver, axdriver_net::fxmac::FXmacNic);

        pub struct FXmacDriver;
        impl DriverProbe for FXmacDriver {
            fn probe_global() -> Option<AxDeviceEnum> {
                info!("fxmac for phytiumpi probe global");
                axdriver_net::fxmac::FXmacNic::init(0).ok().map(AxDeviceEnum::from_net)
            }
        }
    }
}
