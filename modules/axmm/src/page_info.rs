//! A simple physical FrameInfo manager is provided to track and manage
//! the reference count for every 4KB memory page frame in the system.
//!
//! There is a [`FrameInfo`] struct for each physical page frame
//! that keeps track of its reference count.
//! NOTE: If the page is huge page, its [`FrameInfo`] is placed at the
//! starting physical address.
use alloc::boxed::Box;
use core::sync::atomic::{AtomicU8, Ordering};

use lazy_static::lazy_static;
use memory_addr::PhysAddr;

// 4 kb page
const FRAME_SHIFT: usize = 12;

pub const MAX_FRAME_NUM: usize = axconfig::plat::PHYS_MEMORY_SIZE >> FRAME_SHIFT;

lazy_static! {
    static ref FRAME_INFO_TABLE: FrameInfoTable = FrameInfoTable::default();
}

pub fn frame_table() -> &'static FrameInfoTable {
    &FRAME_INFO_TABLE
}

#[derive(Default)]
#[repr(transparent)]
pub(crate) struct FrameInfo {
    ref_count: AtomicU8,
}

pub struct FrameInfoTable {
    data: Box<[FrameInfo; MAX_FRAME_NUM]>,
}

impl Default for FrameInfoTable {
    fn default() -> Self {
        let mut data = Box::new_uninit();
        unsafe {
            core::ptr::write_bytes(data.as_mut_ptr(), 0, 1);
        }
        FrameInfoTable {
            data: unsafe { data.assume_init() },
        }
    }
}

impl FrameInfoTable {
    fn info(&self, paddr: PhysAddr) -> &FrameInfo {
        let index = (paddr.as_usize() - axconfig::plat::PHYS_MEMORY_BASE) >> FRAME_SHIFT;
        &self.data[index]
    }

    /// Increases the reference count of the frame associated with a physical
    /// address.
    ///
    /// # Parameters
    /// - `paddr`: It must be an aligned physical address; if it's a huge page,
    ///   it must be the starting physical address.
    pub fn inc_ref(&self, paddr: PhysAddr) {
        self.info(paddr)
            .ref_count
            .fetch_update(Ordering::Release, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .expect("frame reference overflow");
    }

    /// Decreases the reference count of the frame associated with a physical
    /// address.
    ///
    /// - `paddr`: It must be an aligned physical address; if it's a huge page,
    ///   it must be the starting physical address.
    pub fn dec_ref(&self, paddr: PhysAddr) -> usize {
        self.info(paddr).ref_count.fetch_sub(1, Ordering::AcqRel) as usize
    }
}
