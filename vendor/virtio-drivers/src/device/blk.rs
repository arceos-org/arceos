//! Driver for VirtIO block devices.

use crate::hal::Hal;
use crate::queue::VirtQueue;
use crate::transport::Transport;
use crate::volatile::{volread, Volatile};
use crate::{Error, Result};
use bitflags::bitflags;
use log::info;
use zerocopy::{AsBytes, FromBytes};

const QUEUE: u16 = 0;
const QUEUE_SIZE: u16 = 16;

/// Driver for a VirtIO block device.
///
/// This is a simple virtual block device, e.g. disk.
///
/// Read and write requests (and other exotic requests) are placed in the queue and serviced
/// (probably out of order) by the device except where noted.
///
/// # Example
///
/// ```
/// # use virtio_drivers::{Error, Hal};
/// # use virtio_drivers::transport::Transport;
/// use virtio_drivers::device::blk::{VirtIOBlk, SECTOR_SIZE};
///
/// # fn example<HalImpl: Hal, T: Transport>(transport: T) -> Result<(), Error> {
/// let mut disk = VirtIOBlk::<HalImpl, _>::new(transport)?;
///
/// println!("VirtIO block device: {} kB", disk.capacity() * SECTOR_SIZE as u64 / 2);
///
/// // Read sector 0 and then copy it to sector 1.
/// let mut buf = [0; SECTOR_SIZE];
/// disk.read_block(0, &mut buf)?;
/// disk.write_block(1, &buf)?;
/// # Ok(())
/// # }
/// ```
pub struct VirtIOBlk<H: Hal, T: Transport> {
    transport: T,
    queue: VirtQueue<H, { QUEUE_SIZE as usize }>,
    capacity: u64,
    readonly: bool,
}

impl<H: Hal, T: Transport> VirtIOBlk<H, T> {
    /// Create a new VirtIO-Blk driver.
    pub fn new(mut transport: T) -> Result<Self> {
        let mut readonly = false;

        transport.begin_init(|features| {
            let features = BlkFeature::from_bits_truncate(features);
            info!("device features: {:?}", features);
            readonly = features.contains(BlkFeature::RO);
            // negotiate these flags only
            let supported_features = BlkFeature::empty();
            (features & supported_features).bits()
        });

        // read configuration space
        let config = transport.config_space::<BlkConfig>()?;
        info!("config: {:?}", config);
        // Safe because config is a valid pointer to the device configuration space.
        let capacity = unsafe {
            volread!(config, capacity_low) as u64 | (volread!(config, capacity_high) as u64) << 32
        };
        info!("found a block device of size {}KB", capacity / 2);

        let queue = VirtQueue::new(&mut transport, QUEUE)?;
        transport.finish_init();

        Ok(VirtIOBlk {
            transport,
            queue,
            capacity,
            readonly,
        })
    }

    /// Gets the capacity of the block device, in 512 byte ([`SECTOR_SIZE`]) sectors.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns true if the block device is read-only, or false if it allows writes.
    pub fn readonly(&self) -> bool {
        self.readonly
    }

    /// Acknowledges a pending interrupt, if any.
    ///
    /// Returns true if there was an interrupt to acknowledge.
    pub fn ack_interrupt(&mut self) -> bool {
        self.transport.ack_interrupt()
    }

    /// Reads a block into the given buffer.
    ///
    /// Blocks until the read completes or there is an error.
    pub fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result {
        assert_eq!(buf.len(), SECTOR_SIZE);
        let req = BlkReq {
            type_: ReqType::In,
            reserved: 0,
            sector: block_id as u64,
        };
        let mut resp = BlkResp::default();
        self.queue.add_notify_wait_pop(
            &[req.as_bytes()],
            &mut [buf, resp.as_bytes_mut()],
            &mut self.transport,
        )?;
        resp.status.into()
    }

    /// Submits a request to read a block, but returns immediately without waiting for the read to
    /// complete.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The identifier of the block to read.
    /// * `req` - A buffer which the driver can use for the request to send to the device. The
    ///   contents don't matter as `read_block_nb` will initialise it, but like the other buffers it
    ///   needs to be valid (and not otherwise used) until the corresponding `complete_read_block`
    ///   call.
    /// * `buf` - The buffer in memory into which the block should be read.
    /// * `resp` - A mutable reference to a variable provided by the caller
    ///   to contain the status of the request. The caller can safely
    ///   read the variable only after the request is complete.
    ///
    /// # Usage
    ///
    /// It will submit request to the VirtIO block device and return a token identifying
    /// the position of the first Descriptor in the chain. If there are not enough
    /// Descriptors to allocate, then it returns [`Error::QueueFull`].
    ///
    /// The caller can then call `peek_used` with the returned token to check whether the device has
    /// finished handling the request. Once it has, the caller must call `complete_read_block` with
    /// the same buffers before reading the response.
    ///
    /// ```
    /// # use virtio_drivers::{Error, Hal};
    /// # use virtio_drivers::device::blk::VirtIOBlk;
    /// # use virtio_drivers::transport::Transport;
    /// use virtio_drivers::device::blk::{BlkReq, BlkResp, RespStatus};
    ///
    /// # fn example<H: Hal, T: Transport>(blk: &mut VirtIOBlk<H, T>) -> Result<(), Error> {
    /// let mut request = BlkReq::default();
    /// let mut buffer = [0; 512];
    /// let mut response = BlkResp::default();
    /// let token = unsafe { blk.read_block_nb(42, &mut request, &mut buffer, &mut response) }?;
    ///
    /// // Wait for an interrupt to tell us that the request completed...
    /// assert_eq!(blk.peek_used(), Some(token));
    ///
    /// unsafe {
    ///   blk.complete_read_block(token, &request, &mut buffer, &mut response)?;
    /// }
    /// if response.status() == RespStatus::OK {
    ///   println!("Successfully read block.");
    /// } else {
    ///   println!("Error {:?} reading block.", response.status());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Safety
    ///
    /// `req`, `buf` and `resp` are still borrowed by the underlying VirtIO block device even after
    /// this method returns. Thus, it is the caller's responsibility to guarantee that they are not
    /// accessed before the request is completed in order to avoid data races.
    pub unsafe fn read_block_nb(
        &mut self,
        block_id: usize,
        req: &mut BlkReq,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> Result<u16> {
        assert_eq!(buf.len(), SECTOR_SIZE);
        *req = BlkReq {
            type_: ReqType::In,
            reserved: 0,
            sector: block_id as u64,
        };
        let token = self
            .queue
            .add(&[req.as_bytes()], &mut [buf, resp.as_bytes_mut()])?;
        if self.queue.should_notify() {
            self.transport.notify(QUEUE);
        }
        Ok(token)
    }

    /// Completes a read operation which was started by `read_block_nb`.
    ///
    /// # Safety
    ///
    /// The same buffers must be passed in again as were passed to `read_block_nb` when it returned
    /// the token.
    pub unsafe fn complete_read_block(
        &mut self,
        token: u16,
        req: &BlkReq,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> Result<()> {
        self.queue
            .pop_used(token, &[req.as_bytes()], &mut [buf, resp.as_bytes_mut()])?;
        resp.status.into()
    }

    /// Writes the contents of the given buffer to a block.
    ///
    /// Blocks until the write is complete or there is an error.
    pub fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result {
        assert_eq!(buf.len(), SECTOR_SIZE);
        let req = BlkReq {
            type_: ReqType::Out,
            reserved: 0,
            sector: block_id as u64,
        };
        let mut resp = BlkResp::default();
        self.queue.add_notify_wait_pop(
            &[req.as_bytes(), buf],
            &mut [resp.as_bytes_mut()],
            &mut self.transport,
        )?;
        resp.status.into()
    }

    /// Submits a request to write a block, but returns immediately without waiting for the write to
    /// complete.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The identifier of the block to write.
    /// * `req` - A buffer which the driver can use for the request to send to the device. The
    ///   contents don't matter as `read_block_nb` will initialise it, but like the other buffers it
    ///   needs to be valid (and not otherwise used) until the corresponding `complete_read_block`
    ///   call.
    /// * `buf` - The buffer in memory containing the data to write to the block.
    /// * `resp` - A mutable reference to a variable provided by the caller
    ///   to contain the status of the request. The caller can safely
    ///   read the variable only after the request is complete.
    ///
    /// # Usage
    ///
    /// See [VirtIOBlk::read_block_nb].
    ///
    /// # Safety
    ///
    /// See  [VirtIOBlk::read_block_nb].
    pub unsafe fn write_block_nb(
        &mut self,
        block_id: usize,
        req: &mut BlkReq,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> Result<u16> {
        assert_eq!(buf.len(), SECTOR_SIZE);
        *req = BlkReq {
            type_: ReqType::Out,
            reserved: 0,
            sector: block_id as u64,
        };
        let token = self
            .queue
            .add(&[req.as_bytes(), buf], &mut [resp.as_bytes_mut()])?;
        if self.queue.should_notify() {
            self.transport.notify(QUEUE);
        }
        Ok(token)
    }

    /// Completes a write operation which was started by `write_block_nb`.
    ///
    /// # Safety
    ///
    /// The same buffers must be passed in again as were passed to `write_block_nb` when it returned
    /// the token.
    pub unsafe fn complete_write_block(
        &mut self,
        token: u16,
        req: &BlkReq,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> Result<()> {
        self.queue
            .pop_used(token, &[req.as_bytes(), buf], &mut [resp.as_bytes_mut()])?;
        resp.status.into()
    }

    /// Fetches the token of the next completed request from the used ring and returns it, without
    /// removing it from the used ring. If there are no pending completed requests returns `None`.
    pub fn peek_used(&mut self) -> Option<u16> {
        self.queue.peek_used()
    }

    /// Returns the size of the device's VirtQueue.
    ///
    /// This can be used to tell the caller how many channels to monitor on.
    pub fn virt_queue_size(&self) -> u16 {
        QUEUE_SIZE
    }
}

impl<H: Hal, T: Transport> Drop for VirtIOBlk<H, T> {
    fn drop(&mut self) {
        // Clear any pointers pointing to DMA regions, so the device doesn't try to access them
        // after they have been freed.
        self.transport.queue_unset(QUEUE);
    }
}

#[repr(C)]
struct BlkConfig {
    /// Number of 512 Bytes sectors
    capacity_low: Volatile<u32>,
    capacity_high: Volatile<u32>,
    size_max: Volatile<u32>,
    seg_max: Volatile<u32>,
    cylinders: Volatile<u16>,
    heads: Volatile<u8>,
    sectors: Volatile<u8>,
    blk_size: Volatile<u32>,
    physical_block_exp: Volatile<u8>,
    alignment_offset: Volatile<u8>,
    min_io_size: Volatile<u16>,
    opt_io_size: Volatile<u32>,
    // ... ignored
}

/// A VirtIO block device request.
#[repr(C)]
#[derive(AsBytes, Debug)]
pub struct BlkReq {
    type_: ReqType,
    reserved: u32,
    sector: u64,
}

impl Default for BlkReq {
    fn default() -> Self {
        Self {
            type_: ReqType::In,
            reserved: 0,
            sector: 0,
        }
    }
}

/// Response of a VirtIOBlk request.
#[repr(C)]
#[derive(AsBytes, Debug, FromBytes)]
pub struct BlkResp {
    status: RespStatus,
}

impl BlkResp {
    /// Return the status of a VirtIOBlk request.
    pub fn status(&self) -> RespStatus {
        self.status
    }
}

#[repr(u32)]
#[derive(AsBytes, Debug)]
enum ReqType {
    In = 0,
    Out = 1,
    Flush = 4,
    Discard = 11,
    WriteZeroes = 13,
}

/// Status of a VirtIOBlk request.
#[repr(transparent)]
#[derive(AsBytes, Copy, Clone, Debug, Eq, FromBytes, PartialEq)]
pub struct RespStatus(u8);

impl RespStatus {
    /// Ok.
    pub const OK: RespStatus = RespStatus(0);
    /// IoErr.
    pub const IO_ERR: RespStatus = RespStatus(1);
    /// Unsupported yet.
    pub const UNSUPPORTED: RespStatus = RespStatus(2);
    /// Not ready.
    pub const NOT_READY: RespStatus = RespStatus(3);
}

impl From<RespStatus> for Result {
    fn from(status: RespStatus) -> Self {
        match status {
            RespStatus::OK => Ok(()),
            RespStatus::IO_ERR => Err(Error::IoError),
            RespStatus::UNSUPPORTED => Err(Error::Unsupported),
            RespStatus::NOT_READY => Err(Error::NotReady),
            _ => Err(Error::IoError),
        }
    }
}

impl Default for BlkResp {
    fn default() -> Self {
        BlkResp {
            status: RespStatus::NOT_READY,
        }
    }
}

/// The standard sector size of a VirtIO block device. Data is read and written in multiples of this
/// size.
pub const SECTOR_SIZE: usize = 512;

bitflags! {
    struct BlkFeature: u64 {
        /// Device supports request barriers. (legacy)
        const BARRIER       = 1 << 0;
        /// Maximum size of any single segment is in `size_max`.
        const SIZE_MAX      = 1 << 1;
        /// Maximum number of segments in a request is in `seg_max`.
        const SEG_MAX       = 1 << 2;
        /// Disk-style geometry specified in geometry.
        const GEOMETRY      = 1 << 4;
        /// Device is read-only.
        const RO            = 1 << 5;
        /// Block size of disk is in `blk_size`.
        const BLK_SIZE      = 1 << 6;
        /// Device supports scsi packet commands. (legacy)
        const SCSI          = 1 << 7;
        /// Cache flush command support.
        const FLUSH         = 1 << 9;
        /// Device exports information on optimal I/O alignment.
        const TOPOLOGY      = 1 << 10;
        /// Device can toggle its cache between writeback and writethrough modes.
        const CONFIG_WCE    = 1 << 11;
        /// Device can support discard command, maximum discard sectors size in
        /// `max_discard_sectors` and maximum discard segment number in
        /// `max_discard_seg`.
        const DISCARD       = 1 << 13;
        /// Device can support write zeroes command, maximum write zeroes sectors
        /// size in `max_write_zeroes_sectors` and maximum write zeroes segment
        /// number in `max_write_zeroes_seg`.
        const WRITE_ZEROES  = 1 << 14;

        // device independent
        const NOTIFY_ON_EMPTY       = 1 << 24; // legacy
        const ANY_LAYOUT            = 1 << 27; // legacy
        const RING_INDIRECT_DESC    = 1 << 28;
        const RING_EVENT_IDX        = 1 << 29;
        const UNUSED                = 1 << 30; // legacy
        const VERSION_1             = 1 << 32; // detect legacy

        // the following since virtio v1.1
        const ACCESS_PLATFORM       = 1 << 33;
        const RING_PACKED           = 1 << 34;
        const IN_ORDER              = 1 << 35;
        const ORDER_PLATFORM        = 1 << 36;
        const SR_IOV                = 1 << 37;
        const NOTIFICATION_DATA     = 1 << 38;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hal::fake::FakeHal,
        transport::{
            fake::{FakeTransport, QueueStatus, State},
            DeviceStatus, DeviceType,
        },
    };
    use alloc::{sync::Arc, vec};
    use core::{mem::size_of, ptr::NonNull};
    use std::{sync::Mutex, thread, time::Duration};

    #[test]
    fn config() {
        let mut config_space = BlkConfig {
            capacity_low: Volatile::new(0x42),
            capacity_high: Volatile::new(0x02),
            size_max: Volatile::new(0),
            seg_max: Volatile::new(0),
            cylinders: Volatile::new(0),
            heads: Volatile::new(0),
            sectors: Volatile::new(0),
            blk_size: Volatile::new(0),
            physical_block_exp: Volatile::new(0),
            alignment_offset: Volatile::new(0),
            min_io_size: Volatile::new(0),
            opt_io_size: Volatile::new(0),
        };
        let state = Arc::new(Mutex::new(State {
            status: DeviceStatus::empty(),
            driver_features: 0,
            guest_page_size: 0,
            interrupt_pending: false,
            queues: vec![QueueStatus::default(); 1],
        }));
        let transport = FakeTransport {
            device_type: DeviceType::Console,
            max_queue_size: QUEUE_SIZE.into(),
            device_features: BlkFeature::RO.bits(),
            config_space: NonNull::from(&mut config_space),
            state: state.clone(),
        };
        let blk = VirtIOBlk::<FakeHal, FakeTransport<BlkConfig>>::new(transport).unwrap();

        assert_eq!(blk.capacity(), 0x02_0000_0042);
        assert_eq!(blk.readonly(), true);
    }

    #[test]
    fn read() {
        let mut config_space = BlkConfig {
            capacity_low: Volatile::new(66),
            capacity_high: Volatile::new(0),
            size_max: Volatile::new(0),
            seg_max: Volatile::new(0),
            cylinders: Volatile::new(0),
            heads: Volatile::new(0),
            sectors: Volatile::new(0),
            blk_size: Volatile::new(0),
            physical_block_exp: Volatile::new(0),
            alignment_offset: Volatile::new(0),
            min_io_size: Volatile::new(0),
            opt_io_size: Volatile::new(0),
        };
        let state = Arc::new(Mutex::new(State {
            status: DeviceStatus::empty(),
            driver_features: 0,
            guest_page_size: 0,
            interrupt_pending: false,
            queues: vec![QueueStatus::default(); 1],
        }));
        let transport = FakeTransport {
            device_type: DeviceType::Console,
            max_queue_size: QUEUE_SIZE.into(),
            device_features: 0,
            config_space: NonNull::from(&mut config_space),
            state: state.clone(),
        };
        let mut blk = VirtIOBlk::<FakeHal, FakeTransport<BlkConfig>>::new(transport).unwrap();

        // Start a thread to simulate the device waiting for a read request.
        let handle = thread::spawn(move || {
            println!("Device waiting for a request.");
            while !state.lock().unwrap().queues[usize::from(QUEUE)].notified {
                thread::sleep(Duration::from_millis(10));
            }
            println!("Transmit queue was notified.");

            state
                .lock()
                .unwrap()
                .read_write_queue::<{ QUEUE_SIZE as usize }>(QUEUE, |request| {
                    assert_eq!(
                        request,
                        BlkReq {
                            type_: ReqType::In,
                            reserved: 0,
                            sector: 42
                        }
                        .as_bytes()
                    );

                    let mut response = vec![0; SECTOR_SIZE];
                    response[0..9].copy_from_slice(b"Test data");
                    response.extend_from_slice(
                        BlkResp {
                            status: RespStatus::OK,
                        }
                        .as_bytes(),
                    );

                    response
                });
        });

        // Read a block from the device.
        let mut buffer = [0; 512];
        blk.read_block(42, &mut buffer).unwrap();
        assert_eq!(&buffer[0..9], b"Test data");

        handle.join().unwrap();
    }

    #[test]
    fn write() {
        let mut config_space = BlkConfig {
            capacity_low: Volatile::new(66),
            capacity_high: Volatile::new(0),
            size_max: Volatile::new(0),
            seg_max: Volatile::new(0),
            cylinders: Volatile::new(0),
            heads: Volatile::new(0),
            sectors: Volatile::new(0),
            blk_size: Volatile::new(0),
            physical_block_exp: Volatile::new(0),
            alignment_offset: Volatile::new(0),
            min_io_size: Volatile::new(0),
            opt_io_size: Volatile::new(0),
        };
        let state = Arc::new(Mutex::new(State {
            status: DeviceStatus::empty(),
            driver_features: 0,
            guest_page_size: 0,
            interrupt_pending: false,
            queues: vec![QueueStatus::default(); 1],
        }));
        let transport = FakeTransport {
            device_type: DeviceType::Console,
            max_queue_size: QUEUE_SIZE.into(),
            device_features: 0,
            config_space: NonNull::from(&mut config_space),
            state: state.clone(),
        };
        let mut blk = VirtIOBlk::<FakeHal, FakeTransport<BlkConfig>>::new(transport).unwrap();

        // Start a thread to simulate the device waiting for a write request.
        let handle = thread::spawn(move || {
            println!("Device waiting for a request.");
            while !state.lock().unwrap().queues[usize::from(QUEUE)].notified {
                thread::sleep(Duration::from_millis(10));
            }
            println!("Transmit queue was notified.");

            state
                .lock()
                .unwrap()
                .read_write_queue::<{ QUEUE_SIZE as usize }>(QUEUE, |request| {
                    assert_eq!(
                        &request[0..size_of::<BlkReq>()],
                        BlkReq {
                            type_: ReqType::Out,
                            reserved: 0,
                            sector: 42
                        }
                        .as_bytes()
                    );
                    let data = &request[size_of::<BlkReq>()..];
                    assert_eq!(data.len(), SECTOR_SIZE);
                    assert_eq!(&data[0..9], b"Test data");

                    let mut response = Vec::new();
                    response.extend_from_slice(
                        BlkResp {
                            status: RespStatus::OK,
                        }
                        .as_bytes(),
                    );

                    response
                });
        });

        // Write a block to the device.
        let mut buffer = [0; 512];
        buffer[0..9].copy_from_slice(b"Test data");
        blk.write_block(42, &mut buffer).unwrap();

        handle.join().unwrap();
    }
}
