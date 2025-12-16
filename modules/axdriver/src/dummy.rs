//! Dummy types used if no device of a certain category is selected.

#![allow(unused_imports)]
#![allow(dead_code)]

use super::prelude::*;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(net_dev = "dummy")] {
        use axdriver_net::{EthernetAddress, NetBuf, NetBufBox, NetBufPool, NetBufPtr};

        pub struct DummyNetDev;
        pub struct DummyNetDrvier;
        register_net_driver!(DummyNetDriver, DummyNetDev);

        impl BaseDriverOps for DummyNetDev {
            fn device_type(&self) -> DeviceType { DeviceType::Net }
            fn device_name(&self) -> &str { "dummy-net" }
        }

        impl NetDriverOps for DummyNetDev {
            fn mac_address(&self) -> EthernetAddress { unreachable!() }
            fn can_transmit(&self) -> bool { false }
            fn can_receive(&self) -> bool { false }
            fn rx_queue_size(&self) -> usize { 0 }
            fn tx_queue_size(&self) -> usize { 0 }
            fn recycle_rx_buffer(&mut self, _: NetBufPtr) -> DevResult { Err(DevError::Unsupported) }
            fn recycle_tx_buffers(&mut self) -> DevResult { Err(DevError::Unsupported) }
            fn transmit(&mut self, _: NetBufPtr) -> DevResult { Err(DevError::Unsupported) }
            fn receive(&mut self) -> DevResult<NetBufPtr> { Err(DevError::Unsupported) }
            fn alloc_tx_buffer(&mut self, _: usize) -> DevResult<NetBufPtr> { Err(DevError::Unsupported) }
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
            fn info(&self) -> axdriver_display::DisplayInfo {
                unreachable!()
            }
            fn fb(&self) -> axdriver_display::FrameBuffer<'_> {
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

cfg_if! {
    if #[cfg(input_dev = "dummy")] {
        pub struct DummyInputDev;
        pub struct DummyInputDriver;
        register_input_driver!(DummyInputDriver, DummyInputDev);

        impl BaseDriverOps for DummyInputDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Input
            }
            fn device_name(&self) -> &str {
                "dummy-input"
            }
        }

        impl InputDriverOps for DummyInputDev {
            fn device_id(&self) -> InputDeviceId {
                InputDeviceId { bus_type: 0, vendor: 0, product: 0, version: 0 }
            }
            fn physical_location(&self) -> &str {
                "dummy"
            }
            fn unique_id(&self) -> &str {
                "dummy"
            }
            fn get_event_bits(&mut self, _ty: EventType, _out: &mut [u8]) -> DevResult<bool> {
                Err(DevError::Unsupported)
            }
            fn read_event(&mut self) -> DevResult<Event> {
                Err(DevError::Unsupported)
            }
        }
    }
}

cfg_if! {
    if #[cfg(vsock_dev = "dummy")] {
        pub struct DummyVsockDev;
        pub struct DummyVsockDriver;
        register_vsock_driver!(DummyVsockDriver, DummyVsockDev);

        impl BaseDriverOps for DummyVsockDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Vsock
            }
            fn device_name(&self) -> &str {
                "dummy-vsock"
            }
        }

        impl VsockDriverOps for DummyVsockDev {
            fn guest_cid(&self) -> u64 {
                unimplemented!()
            }
            fn listen(&mut self, _src_port: u32) {
                unimplemented!()
            }
            fn connect(&mut self, _cid: VsockConnId) -> DevResult<()> {
                Err(DevError::Unsupported)
            }
            fn send(&mut self, _cid: VsockConnId, _buf: &[u8]) -> DevResult<usize> {
                Err(DevError::Unsupported)
            }
            fn recv(&mut self, _cid: VsockConnId, _buf: &mut [u8]) -> DevResult<usize> {
                Err(DevError::Unsupported)
            }
            fn recv_avail(&mut self, _cid: VsockConnId) -> DevResult<usize> {
                Err(DevError::Unsupported)
            }
            fn disconnect(&mut self, _cid: VsockConnId) -> DevResult<()> {
                Err(DevError::Unsupported)
            }
            fn abort(&mut self, _cid: VsockConnId) -> DevResult<()> {
                Err(DevError::Unsupported)
            }
            fn poll_event(&mut self, _buf: &mut [u8]) -> DevResult<Option<VsockDriverEvent>> {
                Err(DevError::Unsupported)
            }
        }
    }
}
