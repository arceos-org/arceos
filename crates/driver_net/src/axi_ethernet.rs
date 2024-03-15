use core::convert::From;
use core::pin::Pin;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec;
use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};
use axi_dma::{AxiDma, Transfer};
use axi_ethernet::*;
use spin::Mutex;

use crate::{EthernetAddress, NetBufPtr, NetDriverOps};

extern crate alloc;

const BD_CNT: usize = 1024;
const MAC_ADDR: [u8; 6] = [0x00, 0x0A, 0x35, 0x01, 0x02, 0x03];
const RX_BUFFER_SIZE: usize = 9000;

/// The Axi Ethernet device driver
pub struct AxiNic {
    dma: Arc<AxiDma>,
    eth: Arc<Mutex<AxiEthernet>>,
    tx_transfers: VecDeque<Transfer<Box<[u8]>>>,
    rx_transfers: VecDeque<Transfer<Box<[u8]>>>,
}

impl AxiNic {
    /// Creates a net Axi NIC instance and initialize, or returns a error if
    /// any step fails.
    pub fn init(eth_base: usize, dma_base: usize) -> DevResult<Self> {
        let dma = Arc::new(AxiDma::default());

        dma.reset().map_err(|_| DevError::ResourceBusy)?;
        // enable cyclic mode
        dma.cyclic_enable();
        // init cyclic block descriptor
        dma.tx_channel_create(BD_CNT).map_err(|_| DevError::Unsupported)?;
        dma.rx_channel_create(BD_CNT).map_err(|_| DevError::Unsupported)?;
        // enable tx & rx intr
        dma.intr_enable();

        let slice = vec![0u8; RX_BUFFER_SIZE].into_boxed_slice();
        let rx_buf = Pin::new(slice);
        let rx_transfer = dma.rx_submit(rx_buf).map_err(|_| panic!("Unexpected error"))?;
        let mut rx_transfers = VecDeque::new();
        rx_transfers.push_back(rx_transfer);

        let eth = Arc::new(Mutex::new(AxiEthernet::new(eth_base, dma_base)));

        let mut eth_inner = eth.lock();
        eth_inner.reset();
        let options = eth_inner.get_options();
        eth_inner.set_options(options | XAE_JUMBO_OPTION);
        eth_inner.detect_phy();
        let speed = eth_inner.get_phy_speed_ksz9031();
        log::debug!("speed is: {}", speed);
        eth_inner.set_operating_speed(speed as u16);
        if speed == 0 {
            eth_inner.link_status = LinkStatus::EthLinkDown;
        } else {
            eth_inner.link_status = LinkStatus::EthLinkUp;
        }
        eth_inner.set_mac_address(&MAC_ADDR);
        log::debug!("link_status: {:?}", eth_inner.link_status);
        eth_inner.enable_rx_memovr();
        eth_inner.enable_rx_rject();
        eth_inner.enable_rx_cmplt();
        eth_inner.start();
        drop(eth_inner);
        Ok(Self { 
            dma, 
            eth,
            tx_transfers: VecDeque::new(),
            rx_transfers,
        })
    }
}

impl BaseDriverOps for AxiNic {
    fn device_name(&self) -> &str {
        "axi-ethernet"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl NetDriverOps for AxiNic {
    fn mac_address(&self) -> EthernetAddress {
        let mut mac_address = [0; 6];
        self.eth.lock().get_mac_address(&mut mac_address);
        EthernetAddress(mac_address)
    }

    fn rx_queue_size(&self) -> usize {
        0x8000
    }

    fn tx_queue_size(&self) -> usize {
        0x8000
    }

    fn can_receive(&self) -> bool {
        self.eth.lock().is_ready()
    }

    fn can_transmit(&self) -> bool {
        // Default implementation is return true forever.
        self.eth.lock().is_ready()
    }

    fn recycle_rx_buffer(&mut self, rx_buf: NetBufPtr) -> DevResult {
        let rx_buf = buf_to_pin_buf(rx_buf)?;
        drop(rx_buf);
        Ok(())
    }

    fn recycle_tx_buffers(&mut self) -> DevResult {
        self.tx_transfers.pop_front().unwrap();
        Ok(())
    }

    fn receive(&mut self) -> DevResult<NetBufPtr> {
        if !self.can_receive() {
            return Err(DevError::Again);
        }
        if let Some(transfer) = self.rx_transfers.pop_front() {
            let buf = transfer.wait().map_err(|_| panic!("Unexpected error"))?;
            Ok(NetBufPtr::from(buf))
        } else {
            // RX queue is empty, receive from AxiNIC.
            let slice = vec![0u8; RX_BUFFER_SIZE].into_boxed_slice();
            let rx_buf = Pin::new(slice);
            let completed_buf = self.dma.rx_submit(rx_buf).map_err(|_| panic!("Unexpected error"))?.wait().unwrap();
            Ok(NetBufPtr::from(completed_buf))
        }
    }

    fn transmit(&mut self, tx_buf: NetBufPtr) -> DevResult {
        let tx_buf = buf_to_pin_buf(tx_buf)?;
        match self.dma.tx_submit(tx_buf) {
            Ok(transfer) => {
                self.tx_transfers.push_back(transfer);
                Ok(())
            },
            Err(err) => panic!("Unexpected err: {:?}", err),
        }
        
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<NetBufPtr> {
        let slice = vec![0u8; size].into_boxed_slice();
        let tx_buf = Pin::new(slice);
        Ok(NetBufPtr::from(tx_buf))
    }
}

impl From<Pin<Box<[u8]>>> for NetBufPtr {
    fn from(value: Pin<Box<[u8]>>) -> Self {
        let raw_buf = Pin::into_inner(value);
        let len = raw_buf.len();
        let buf_ptr = Box::into_raw(raw_buf);
        Self { 
            raw_ptr: NonNull::new(core::ptr::null_mut()).unwrap(), 
            buf_ptr: NonNull::new(buf_ptr as *mut u8).unwrap(), 
            len 
        }

    }
}

// Converts a `NetBufPtr` to `Pin<Box<B>>`.
fn buf_to_pin_buf(ptr: NetBufPtr) -> DevResult<Pin<Box<[u8]>>> {
    let buf_ptr = ptr.buf_ptr.as_ptr();
    let len = ptr.len;
    let raw_buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, len) };
    let buf = Pin::new(unsafe { Box::from_raw(raw_buf) });
    Ok(buf)
}

