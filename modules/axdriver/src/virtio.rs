use core::ptr::NonNull;

use axalloc::global_allocator;
use axhal::mem::{phys_to_virt, virt_to_phys};
use cfg_if::cfg_if;
use driver_common::{BaseDriverOps, DeviceType};
use driver_virtio::{BufferDirection, PhysAddr, VirtIoHal};

use crate::AllDevices;

cfg_if! {
    if #[cfg(feature =  "bus-mmio")] {
        type VirtIoTransport = driver_virtio::MmioTransport;
    } else if #[cfg(feature = "bus-pci")] {
        type VirtIoTransport = driver_virtio::PciTransport;
    }
}

cfg_if! {
    if #[cfg(feature = "virtio-blk")] {
        pub type VirtIoBlockDev = driver_virtio::VirtIoBlkDev<VirtIoHalImpl, VirtIoTransport>;
    }
}

cfg_if! {
    if #[cfg(feature = "virtio-net")] {
        const NET_QUEUE_SIZE: usize = 64;
        const NET_BUFFER_SIZE: usize = 2048;
        pub type VirtIoNetDev = driver_virtio::VirtIoNetDev<VirtIoHalImpl, VirtIoTransport, NET_QUEUE_SIZE>;
    }
}

pub struct VirtIoHalImpl;

unsafe impl VirtIoHal for VirtIoHalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let vaddr = if let Ok(vaddr) = global_allocator().alloc_pages(pages, 0x1000) {
            vaddr
        } else {
            return (0, NonNull::dangling());
        };
        let paddr = virt_to_phys(vaddr.into());
        let ptr = NonNull::new(vaddr as _).unwrap();
        (paddr.as_usize(), ptr)
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        global_allocator().dealloc_pages(vaddr.as_ptr() as usize, pages);
        0
    }

    #[inline]
    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(phys_to_virt(paddr.into()).as_mut_ptr()).unwrap()
    }

    #[inline]
    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        virt_to_phys(vaddr.into()).into()
    }

    #[inline]
    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}

impl AllDevices {
    #[cfg(feature = "bus-mmio")]
    fn probe_devices_common<D, F>(dev_type: DeviceType, ret: F) -> Option<D>
    where
        D: BaseDriverOps,
        F: FnOnce(VirtIoTransport) -> Option<D>,
    {
        // TODO: parse device tree
        for reg in axconfig::VIRTIO_MMIO_REGIONS {
            if let Some(transport) = driver_virtio::probe_mmio_device(
                phys_to_virt(reg.0.into()).as_mut_ptr(),
                reg.1,
                Some(dev_type),
            ) {
                let dev = ret(transport)?;
                info!(
                    "created a new {:?} device: {:?}",
                    dev.device_type(),
                    dev.device_name()
                );
                return Some(dev);
            }
        }
        None
    }

    #[cfg(feature = "virtio-blk")]
    pub(crate) fn probe_virtio_blk() -> Option<VirtIoBlockDev> {
        Self::probe_devices_common(DeviceType::Block, |t| VirtIoBlockDev::try_new(t).ok())
    }

    #[cfg(feature = "virtio-net")]
    pub(crate) fn probe_virtio_net() -> Option<VirtIoNetDev> {
        Self::probe_devices_common(DeviceType::Net, |t| {
            VirtIoNetDev::try_new(t, NET_BUFFER_SIZE).ok()
        })
    }
}
