//! TLSF_C memory allocation.
//!
#![feature(allocator_api)]
#![no_std]

extern crate alloc;
use alloc::alloc::AllocError;

use core::ffi::c_ulonglong;

#[link(name = "tlsf")]
extern "C" {
    pub fn tlsf_create_with_pool(mem: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;
    pub fn tlsf_add_pool(tlsf: c_ulonglong, mem: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;

    pub fn tlsf_malloc(tlsf: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong; //申请一段内存
    pub fn tlsf_memalign(tlsf: c_ulonglong, align: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong; //申请一段内存，要求对齐到align
    pub fn tlsf_free(tlsf: c_ulonglong, ptr: c_ulonglong); //回收
}

/// the structure in rust code
pub struct Heap {
    inner: Option<c_ulonglong>,
}

impl Heap {
    pub const fn new() -> Self {
        Self { inner: None }
    }
    pub fn inner_mut(&mut self) -> &mut c_ulonglong {
        self.inner.as_mut().unwrap()
    }

    pub fn inner(&self) -> &c_ulonglong {
        self.inner.as_ref().unwrap()
    }

    pub fn init(&mut self, start: usize, size: usize) {
        unsafe {
            self.inner = Some(
                tlsf_create_with_pool(start as c_ulonglong, size as c_ulonglong) as c_ulonglong,
            );
        }
    }

    pub fn add_memory(&mut self, start: usize, size: usize) {
        unsafe {
            tlsf_add_pool(
                *self.inner() as c_ulonglong,
                start as c_ulonglong,
                size as c_ulonglong,
            );
        }
    }

    pub fn allocate(&mut self, size: usize, align_pow2: usize) -> Result<usize, AllocError> {
        if align_pow2 <= 8 {
            unsafe {
                let ptr = tlsf_malloc(*self.inner() as c_ulonglong, size as c_ulonglong) as usize;
                if ptr == 0 {
                    return Err(AllocError);
                }
                Ok(ptr)
            }
        } else {
            unsafe {
                let ptr = tlsf_memalign(
                    *self.inner() as c_ulonglong,
                    align_pow2 as c_ulonglong,
                    size as c_ulonglong,
                ) as usize;
                if ptr == 0 {
                    return Err(AllocError);
                }
                Ok(ptr)
            }
        }
    }

    pub fn deallocate(&mut self, pos: usize, _size: usize, _align_pow2: usize) {
        unsafe {
            tlsf_free(*self.inner() as c_ulonglong, pos as c_ulonglong);
        }
    }

    pub fn total_bytes(&self) -> usize {
        0
    }

    pub fn used_bytes(&self) -> usize {
        0
    }

    pub fn available_bytes(&self) -> usize {
        0
    }
}
