extern crate alloc;

use crate::{DevError, DevResult, NetBufPtr};
use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use core::ptr::NonNull;
use spin::Mutex;

const MIN_BUFFER_LEN: usize = 1526;
const MAX_BUFFER_LEN: usize = 65535;

/// A RAII network buffer wrapped in a [`Box`].
pub type NetBufBox = Box<NetBuf>;

/// A RAII network buffer.
///
/// It should be allocated from the [`NetBufPool`], and it will be
/// deallocated into the pool automatically when dropped.
///
/// The layout of the buffer is:
///
/// ```text
///   ______________________ capacity ______________________
///  /                                                      \
/// +------------------+------------------+------------------+
/// |      Header      |      Packet      |      Unused      |
/// +------------------+------------------+------------------+
/// |\__ header_len __/ \__ packet_len __/
/// |
/// buf_ptr
/// ```
pub struct NetBuf {
    header_len: usize,
    packet_len: usize,
    capacity: usize,
    buf_ptr: NonNull<u8>,
    pool_offset: usize,
    pool: Arc<NetBufPool>,
}

unsafe impl Send for NetBuf {}
unsafe impl Sync for NetBuf {}

impl NetBuf {
    const unsafe fn get_slice(&self, start: usize, len: usize) -> &[u8] {
        core::slice::from_raw_parts(self.buf_ptr.as_ptr().add(start), len)
    }

    const unsafe fn get_slice_mut(&mut self, start: usize, len: usize) -> &mut [u8] {
        core::slice::from_raw_parts_mut(self.buf_ptr.as_ptr().add(start), len)
    }

    /// Returns the capacity of the buffer.
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the length of the header part.
    pub const fn header_len(&self) -> usize {
        self.header_len
    }

    /// Returns the header part of the buffer.
    pub const fn header(&self) -> &[u8] {
        unsafe { self.get_slice(0, self.header_len) }
    }

    /// Returns the packet part of the buffer.
    pub const fn packet(&self) -> &[u8] {
        unsafe { self.get_slice(self.header_len, self.packet_len) }
    }

    /// Returns the mutable reference to the packet part.
    pub const fn packet_mut(&mut self) -> &mut [u8] {
        unsafe { self.get_slice_mut(self.header_len, self.packet_len) }
    }

    /// Returns both the header and the packet parts, as a contiguous slice.
    pub const fn packet_with_header(&self) -> &[u8] {
        unsafe { self.get_slice(0, self.header_len + self.packet_len) }
    }

    /// Returns the entire buffer.
    pub const fn raw_buf(&self) -> &[u8] {
        unsafe { self.get_slice(0, self.capacity) }
    }

    /// Returns the mutable reference to the entire buffer.
    pub const fn raw_buf_mut(&mut self) -> &mut [u8] {
        unsafe { self.get_slice_mut(0, self.capacity) }
    }

    /// Set the length of the header part.
    pub fn set_header_len(&mut self, header_len: usize) {
        debug_assert!(header_len + self.packet_len <= self.capacity);
        self.header_len = header_len;
    }

    /// Set the length of the packet part.
    pub fn set_packet_len(&mut self, packet_len: usize) {
        debug_assert!(self.header_len + packet_len <= self.capacity);
        self.packet_len = packet_len;
    }

    /// Converts the buffer into a [`NetBufPtr`].
    pub fn into_buf_ptr(mut self: Box<Self>) -> NetBufPtr {
        let buf_ptr = self.packet_mut().as_mut_ptr();
        let len = self.packet_len;
        NetBufPtr::new(
            NonNull::new(Box::into_raw(self) as *mut u8).unwrap(),
            NonNull::new(buf_ptr).unwrap(),
            len,
        )
    }

    /// Restore [`NetBuf`] struct from a raw pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it may cause some memory issues,
    /// so we must ensure that it is called after calling `into_buf_ptr`.
    pub unsafe fn from_buf_ptr(ptr: NetBufPtr) -> Box<Self> {
        Box::from_raw(ptr.raw_ptr::<Self>())
    }
}

impl Drop for NetBuf {
    /// Deallocates the buffer into the [`NetBufPool`].
    fn drop(&mut self) {
        self.pool.dealloc(self.pool_offset);
    }
}

/// A pool of [`NetBuf`]s to speed up buffer allocation.
///
/// It divides a large memory into several equal parts for each buffer.
pub struct NetBufPool {
    capacity: usize,
    buf_len: usize,
    pool: Vec<u8>,
    free_list: Mutex<Vec<usize>>,
}

impl NetBufPool {
    /// Creates a new pool with the given `capacity`, and all buffer lengths are
    /// set to `buf_len`.
    pub fn new(capacity: usize, buf_len: usize) -> DevResult<Arc<Self>> {
        if capacity == 0 {
            return Err(DevError::InvalidParam);
        }
        if !(MIN_BUFFER_LEN..=MAX_BUFFER_LEN).contains(&buf_len) {
            return Err(DevError::InvalidParam);
        }

        let pool = vec![0; capacity * buf_len];
        let mut free_list = Vec::with_capacity(capacity);
        for i in 0..capacity {
            free_list.push(i * buf_len);
        }
        Ok(Arc::new(Self {
            capacity,
            buf_len,
            pool,
            free_list: Mutex::new(free_list),
        }))
    }

    /// Returns the capacity of the pool.
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the length of each buffer.
    pub const fn buffer_len(&self) -> usize {
        self.buf_len
    }

    /// Allocates a buffer from the pool.
    ///
    /// Returns `None` if no buffer is available.
    pub fn alloc(self: &Arc<Self>) -> Option<NetBuf> {
        let pool_offset = self.free_list.lock().pop()?;
        let buf_ptr =
            unsafe { NonNull::new(self.pool.as_ptr().add(pool_offset) as *mut u8).unwrap() };
        Some(NetBuf {
            header_len: 0,
            packet_len: 0,
            capacity: self.buf_len,
            buf_ptr,
            pool_offset,
            pool: Arc::clone(self),
        })
    }

    /// Allocates a buffer wrapped in a [`Box`] from the pool.
    ///
    /// Returns `None` if no buffer is available.
    pub fn alloc_boxed(self: &Arc<Self>) -> Option<NetBufBox> {
        Some(Box::new(self.alloc()?))
    }

    /// Deallocates a buffer at the given offset.
    ///
    /// `pool_offset` must be a multiple of `buf_len`.
    fn dealloc(&self, pool_offset: usize) {
        debug_assert_eq!(pool_offset % self.buf_len, 0);
        self.free_list.lock().push(pool_offset);
    }
}
