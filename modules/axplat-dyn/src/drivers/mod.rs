extern crate alloc;

use alloc::{
    alloc::{alloc, alloc_zeroed, dealloc},
    boxed::Box,
};
use core::ptr::NonNull;

use axdriver_block::BlockDriverOps;
use axerrno::AxError;
use axplat::mem::PhysAddr;
use heapless::Vec;
use memory_addr::PAGE_SIZE_4K;
use rdrive::probe::OnProbeError;
use spin::Mutex;

mod pci;

pub mod blk;

const MAX_BLOCK_DEVICES: usize = 16;

pub type DynBlockDevice = Box<dyn BlockDriverOps>;

static BLOCK_DEVICES: Mutex<Vec<DynBlockDevice, MAX_BLOCK_DEVICES>> = Mutex::new(Vec::new());

pub fn clear_block_devices() {
    BLOCK_DEVICES.lock().clear();
}

pub fn register_block_device(device: DynBlockDevice) -> Result<(), DynBlockDevice> {
    BLOCK_DEVICES.lock().push(device)
}

pub fn take_block_devices() -> Vec<DynBlockDevice, MAX_BLOCK_DEVICES> {
    let mut devices = BLOCK_DEVICES.lock();
    core::mem::take(&mut *devices)
}

/// maps a mmio physical address to a virtual address.
pub(crate) fn iomap(addr: PhysAddr, size: usize) -> Result<NonNull<u8>, OnProbeError> {
    axklib::mem::iomap(addr, size)
        .map_err(|e| match e {
            AxError::NoMemory => OnProbeError::KError(rdrive::KError::NoMem),
            _ => OnProbeError::Other(alloc::format!("{e:?}").into()),
        })
        .map(|v| unsafe { NonNull::new_unchecked(v.as_mut_ptr()) })
}

pub fn probe_all_devices() -> Result<(), AxError> {
    clear_block_devices();
    rdrive::probe_all(true).map_err(|_| AxError::BadState)?;

    for dev in rdrive::get_list::<rd_block::Block>() {
        let block = Box::new(blk::Block::from(dev));
        if register_block_device(block).is_err() {
            return Err(AxError::NoMemory);
        }
    }

    Ok(())
}

pub(crate) struct DmaImpl;

#[inline]
fn dma_addr_from_ptr(ptr: NonNull<u8>) -> u64 {
    somehal::mem::virt_to_phys(ptr.as_ptr()) as u64
}

#[inline]
fn dma_range_fits_mask(dma_addr: u64, size: usize, dma_mask: u64) -> bool {
    if size == 0 {
        dma_addr <= dma_mask
    } else {
        dma_addr
            .checked_add(size.saturating_sub(1) as u64)
            .map(|end| end <= dma_mask)
            .unwrap_or(false)
    }
}

#[inline]
fn dma_addr_is_aligned(dma_addr: u64, align: usize) -> bool {
    dma_addr.is_multiple_of(align as u64)
}

impl dma_api::DmaOp for DmaImpl {
    fn page_size(&self) -> usize {
        PAGE_SIZE_4K
    }

    unsafe fn map_single(
        &self,
        dma_mask: u64,
        addr: NonNull<u8>,
        size: core::num::NonZeroUsize,
        align: usize,
        direction: dma_api::DmaDirection,
    ) -> Result<dma_api::DmaMapHandle, dma_api::DmaError> {
        let layout = core::alloc::Layout::from_size_align(size.get(), align)?;
        let dma_addr = dma_addr_from_ptr(addr);

        if dma_range_fits_mask(dma_addr, size.get(), dma_mask)
            && dma_addr_is_aligned(dma_addr, align)
        {
            return Ok(unsafe { dma_api::DmaMapHandle::new(addr, dma_addr.into(), layout, None) });
        }

        let map_ptr = unsafe { alloc(layout) };
        let map_virt = NonNull::new(map_ptr).ok_or(dma_api::DmaError::NoMemory)?;

        if matches!(
            direction,
            dma_api::DmaDirection::ToDevice | dma_api::DmaDirection::Bidirectional
        ) {
            unsafe {
                map_virt
                    .as_ptr()
                    .copy_from_nonoverlapping(addr.as_ptr(), size.get());
            }
        }

        let map_dma_addr = dma_addr_from_ptr(map_virt);
        if !dma_range_fits_mask(map_dma_addr, size.get(), dma_mask) {
            unsafe { dealloc(map_virt.as_ptr(), layout) };
            return Err(dma_api::DmaError::DmaMaskNotMatch {
                addr: map_dma_addr.into(),
                mask: dma_mask,
            });
        }
        if !dma_addr_is_aligned(map_dma_addr, align) {
            unsafe { dealloc(map_virt.as_ptr(), layout) };
            return Err(dma_api::DmaError::AlignMismatch {
                required: align,
                address: map_dma_addr.into(),
            });
        }

        Ok(
            unsafe {
                dma_api::DmaMapHandle::new(addr, map_dma_addr.into(), layout, Some(map_virt))
            },
        )
    }

    unsafe fn unmap_single(&self, handle: dma_api::DmaMapHandle) {
        if let Some(map_virt) = handle.alloc_virt() {
            unsafe { dealloc(map_virt.as_ptr(), handle.layout()) };
        }
    }

    unsafe fn alloc_coherent(
        &self,
        dma_mask: u64,
        layout: core::alloc::Layout,
    ) -> Option<dma_api::DmaHandle> {
        let ptr = unsafe { alloc_zeroed(layout) };
        let cpu_addr = NonNull::new(ptr)?;

        let dma_addr = dma_addr_from_ptr(cpu_addr);
        if !dma_range_fits_mask(dma_addr, layout.size(), dma_mask)
            || !dma_addr_is_aligned(dma_addr, layout.align())
        {
            unsafe { dealloc(cpu_addr.as_ptr(), layout) };
            return None;
        }

        Some(unsafe { dma_api::DmaHandle::new(cpu_addr, dma_addr.into(), layout) })
    }

    unsafe fn dealloc_coherent(&self, handle: dma_api::DmaHandle) {
        unsafe { dealloc(handle.as_ptr().as_ptr(), handle.layout()) };
    }
}
