use alloc::boxed::Box;
use core::ptr::NonNull;

use axalloc::global_allocator;
use axhal::mem::{phys_to_virt, virt_to_phys};
use driver_virtio::{BufferDirection, PhysAddr, VirtIoDevice, VirtIoHal};

use crate::AllDevices;

const VIRTIO_MMIO_REGIONS: &[(usize, usize)] = &[
    (0x1000_1000, 0x1000),
    (0x1000_2000, 0x1000),
    (0x1000_3000, 0x1000),
    (0x1000_4000, 0x1000),
    (0x1000_5000, 0x1000),
    (0x1000_6000, 0x1000),
    (0x1000_7000, 0x1000),
    (0x1000_8000, 0x1000),
];

struct VirtIoHalImpl;

unsafe impl VirtIoHal for VirtIoHalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let vaddr = global_allocator().alloc_pages(pages, 0x1000).unwrap();
        let paddr = virt_to_phys(vaddr.into());
        let ptr = NonNull::new(vaddr as _).unwrap();
        (paddr.as_usize(), ptr)
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        global_allocator().dealloc_pages(vaddr.as_ptr() as usize, pages);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(phys_to_virt(paddr.into()).as_mut_ptr()).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        virt_to_phys(vaddr.into()).into()
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}

impl AllDevices {
    fn prob_virtio_mmio_device(&mut self, reg_base: usize, reg_size: usize) {
        match driver_virtio::new_from_mmio::<VirtIoHalImpl>(
            phys_to_virt(reg_base.into()).as_mut_ptr(),
            reg_size,
        ) {
            #[cfg(feature = "net")]
            Some(VirtIoDevice::Net(dev)) => {
                info!("Added new net device.");
                self.net.push(Box::new(dev));
            }
            _ => {}
        }
    }

    pub(crate) fn prob_virtio_devices(&mut self) {
        // TODO: parse device tree
        for reg in VIRTIO_MMIO_REGIONS {
            self.prob_virtio_mmio_device(reg.0, reg.1);
        }
    }
}
