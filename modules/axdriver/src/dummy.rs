//! Dummy types used if no device of a certain category is selected.

#![allow(unused_imports)]
#![allow(dead_code)]

use super::prelude::*;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(net_dev = "dummy")] {
        use driver_net::{EthernetAddress, NetBuffer, NetBufferBox, NetBufferPool};

        pub struct DummyNetDev;
        pub struct DummyNetDrvier;
        register_net_driver!(DummyNetDriver, DummyNetDev);

        impl BaseDriverOps for DummyNetDev {
            fn device_type(&self) -> DeviceType { DeviceType::Net }
            fn device_name(&self) -> &str { "dummy-net" }
        }

        impl<'a> NetDriverOps<'a> for DummyNetDev {
            fn mac_address(&self) -> EthernetAddress { unreachable!() }
            fn can_transmit(&self) -> bool { false }
            fn can_receive(&self) -> bool { false }
            fn rx_queue_size(&self) -> usize { 0 }
            fn tx_queue_size(&self) -> usize { 0 }
            fn fill_rx_buffers(&mut self, _: &NetBufferPool) -> DevResult { Err(DevError::Unsupported) }
            fn prepare_tx_buffer(&self, _: &mut NetBuffer, _: usize) -> DevResult { Err(DevError::Unsupported) }
            fn recycle_rx_buffer(&mut self, _: NetBufferBox<'a>) -> DevResult { Err(DevError::Unsupported) }
            fn transmit(&mut self, _: &NetBuffer) -> DevResult { Err(DevError::Unsupported) }
            fn receive(&mut self) -> DevResult<NetBufferBox<'a>> { Err(DevError::Unsupported) }
        }
    }
}

cfg_if! {
    if #[cfg(block_dev = "dummy")] {
        pub struct DummyBlockDev;
        pub struct DummyBlockDriver;
        register_block_driver!(DummyBlockDriver, DummyBlockDev);

        impl BaseDriverOps for DummyBlockDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Block
            }
            fn device_name(&self) -> &str {
                "dummy-block"
            }
        }

        impl BlockDriverOps for DummyBlockDev {
            fn num_blocks(&self) -> u64 {
                0
            }
            fn block_size(&self) -> usize {
                0
            }
            fn read_block(&mut self, _: u64, _: &mut [u8]) -> DevResult {
                Err(DevError::Unsupported)
            }
            fn write_block(&mut self, _: u64, _: &[u8]) -> DevResult {
                Err(DevError::Unsupported)
            }
            fn flush(&mut self) -> DevResult {
                Err(DevError::Unsupported)
            }

            fn read_block_nb(
                &mut self,
                _: u64,
                _: &mut virtio_drivers::device::blk::BlkReq,
                _: &mut [u8],
                _: &mut virtio_drivers::device::blk::BlkResp,
            ) -> DevResult<u16> {
                Err(DevError::Unsupported)
            }

            fn write_block_nb(
                &mut self,
                _: u64,
                _: &mut virtio_drivers::device::blk::BlkReq,
                _: &[u8],
                _: &mut virtio_drivers::device::blk::BlkResp,
            ) -> DevResult<u16> {
                Err(DevError::Unsupported)
            }


            fn complete_read_block(
                &mut self,
                _: u16,
                _: &virtio_drivers::device::blk::BlkReq,
                _: &mut [u8],
                _: &mut virtio_drivers::device::blk::BlkResp,
            ) -> DevResult {
                Err(DevError::Unsupported)
            }


            fn complete_write_block(
                &mut self,
                _: u16,
                _: &virtio_drivers::device::blk::BlkReq,
                _: &[u8],
                _: &mut virtio_drivers::device::blk::BlkResp,
            ) -> DevResult {
                Err(DevError::Unsupported)
            }

            fn peek_used(&mut self) -> Option<u16> {
                None
            }
        }
    }
}

cfg_if! {
    if #[cfg(display_dev = "dummy")] {
        pub struct DummyDisplayDev;
        pub struct DummyDisplayDriver;
        register_display_driver!(DummyDisplayDriver, DummyDisplayDev);

        impl BaseDriverOps for DummyDisplayDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Display
            }
            fn device_name(&self) -> &str {
                "dummy-display"
            }
        }

        impl DisplayDriverOps for DummyDisplayDev {
            fn info(&self) -> driver_display::DisplayInfo {
                unreachable!()
            }
            fn fb(&self) -> driver_display::FrameBuffer {
                unreachable!()
            }
            fn need_flush(&self) -> bool {
                false
            }
            fn flush(&mut self) -> DevResult {
                Err(DevError::Unsupported)
            }
        }
    }
}
