use crate::as_dev_err;
use driver_block::BlockDriverOps;
use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
use virtio_drivers::{
    device::blk::{BlkReq, BlkResp, VirtIOBlk as InnerDev},
    transport::Transport,
    Hal,
};

/// The VirtIO block device driver.
pub struct VirtIoBlkDev<H: Hal, T: Transport> {
    inner: InnerDev<H, T>,
    irq_num: Option<usize>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoBlkDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoBlkDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoBlkDev<H, T> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T, irq_num: Option<usize>) -> DevResult<Self> {
        Ok(Self {
            inner: InnerDev::new(transport).map_err(as_dev_err)?,
            irq_num,
        })
    }
}

impl<H: Hal, T: Transport> BaseDriverOps for VirtIoBlkDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-blk"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }

    fn get_irq_num(&self) -> Option<usize> {
        self.irq_num
    }

    fn ack_interrupt(&mut self) -> bool {
        self.inner.ack_interrupt()
    }
}

impl<H: Hal, T: Transport> BlockDriverOps for VirtIoBlkDev<H, T> {
    #[inline]
    fn num_blocks(&self) -> u64 {
        self.inner.capacity()
    }

    #[inline]
    fn block_size(&self) -> usize {
        virtio_drivers::device::blk::SECTOR_SIZE
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
        self.inner
            .read_block(block_id as _, buf)
            .map_err(as_dev_err)
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
        self.inner
            .write_block(block_id as _, buf)
            .map_err(as_dev_err)
    }

    fn read_block_nb(
        &mut self,
        block_id: u64,
        req: &mut BlkReq,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> DevResult<u16> {
        let token = unsafe {
            self.inner
                .read_block_nb(block_id as usize, req, buf, resp)
                .unwrap()
        };
        Ok(token)
        // yield_now();

        // assert_eq!(self.inner.peek_used(), Some(token));

        // unsafe { self.inner.complete_read_block(token, &req, buf, &mut resp) };

        // if resp.status() == RespStatus::OK {
        //     Ok(())
        // } else {
        //     Err(DevError::Io)
        // }
    }

    fn write_block_nb(
        &mut self,
        block_id: u64,
        req: &mut BlkReq,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> DevResult<u16> {
        let token = unsafe {
            self.inner
                .write_block_nb(block_id as usize, req, buf, resp)
                .unwrap()
        };
        // Err(DevError::Again)
        Ok(token)
    }

    fn complete_read_block(
        &mut self,
        token: u16,
        req: &BlkReq,
        buf: &mut [u8],
        resp: &mut BlkResp,
    ) -> DevResult {
        unsafe {
            self.inner
                .complete_read_block(token, req, buf, resp)
                .map_err(|_| DevError::Io)?;
        }
        self.ack_interrupt();
        Ok(())
    }

    fn complete_write_block(
        &mut self,
        token: u16,
        req: &BlkReq,
        buf: &[u8],
        resp: &mut BlkResp,
    ) -> DevResult {
        unsafe {
            self.inner
                .complete_write_block(token, req, buf, resp)
                .map_err(|_| DevError::Io)?;
        }
        self.ack_interrupt();
        Ok(())
    }

    fn peek_used(&mut self) -> Option<u16> {
        self.inner.peek_used()
    }

    fn flush(&mut self) -> DevResult {
        Ok(())
    }
}
