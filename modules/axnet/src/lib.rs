//! [ArceOS](https://github.com/rcore-os/arceos) network module.
//!
//! It provides unified networking primitives for TCP/UDP communication
//! using various underlying network stacks. Currently, only [smoltcp] is
//! supported.
//!
//! # Organization
//!
//! - [`TcpSocket`]: A TCP socket that provides POSIX-like APIs.
//! - [`UdpSocket`]: A UDP socket that provides POSIX-like APIs.
//! - [`IpAddr`], [`Ipv4Addr`]: IP addresses (either v4 or v6) and IPv4 addresses.
//! - [`SocketAddr`]: IP address with a port number.
//! - [`resolve_socket_addr`]: Function for DNS query.
//!
//! # Cargo Features
//!
//! - `smoltcp`: Use [smoltcp] as the underlying network stack. This is enabled
//!   by default.
//!
//! [smoltcp]: https://github.com/smoltcp-rs/smoltcp

#![no_std]
#![feature(new_uninit)]

#[macro_use]
extern crate log;
extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "smoltcp")] {
        mod smoltcp_impl;
        use smoltcp_impl as net_impl;
    }
}

pub use self::net_impl::resolve_socket_addr;
pub use self::net_impl::TcpSocket;
pub use self::net_impl::UdpSocket;
pub use smoltcp::wire::{IpAddress as IpAddr, IpEndpoint as SocketAddr, Ipv4Address as Ipv4Addr};

#[cfg(not(feature = "user"))]
use axdriver::{prelude::*, AxDeviceContainer};

#[cfg(not(feature = "user"))]
/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    let dev = net_devs.take_one().expect("No NIC device found!");
    info!("  use NIC 0: {:?}", dev.device_name());
    net_impl::init(dev);
}

#[cfg(feature = "user")]
mod user {
    use driver_net::{
        DevError, DevResult, EthernetAddress, NetBuffer, NetBufferBox, NetBufferPool,
    };
    use libax::io::File;
    pub type AxNetDevice = AxNetDeviceMock<'static>;

    pub struct AxNetDeviceMock<'a> {
        rx_buffer: Option<NetBufferBox<'a>>,
        daemon_file: File,
    }

    impl<'a> AxNetDeviceMock<'a> {
        pub fn new() -> DevResult<Self> {
            Ok(Self {
                rx_buffer: None,
                daemon_file: File::open("dev:/net/").map_err(|_| DevError::Unsupported)?,
            })
        }

        pub fn mac_address(&self) -> EthernetAddress {
            let file = File::open("dev:/net/addr").unwrap();
            let mut addr = EthernetAddress([0; 6]);
            file.read(&mut addr.0).unwrap();
            addr
        }
        /*
        pub fn can_transmit(&self) -> bool {
            unimplemented!();
        }

        pub fn can_receive(&self) -> bool {
            unimplemented!();
        }

        pub fn rx_queue_size(&self) -> usize {
            unimplemented!();
        }

        pub fn tx_queue_size(&self) -> usize {
            unimplemented!();
        }
         */
        pub fn fill_rx_buffers(&mut self, buf_pool: &'a NetBufferPool) -> DevResult {
            self.rx_buffer = Some(buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?);
            Ok(())
        }

        pub fn prepare_tx_buffer(&self, tx_buf: &mut NetBuffer, packet_len: usize) -> DevResult {
            if packet_len > tx_buf.capacity() {
                return Err(DevError::InvalidParam);
            }
            tx_buf.set_header_len(0);
            tx_buf.set_packet_len(packet_len);
            Ok(())
        }

        pub fn recycle_rx_buffer(&mut self, rx_buf: NetBufferBox<'a>) -> DevResult {
            self.rx_buffer = Some(rx_buf);
            Ok(())
        }

        pub fn transmit(&mut self, tx_buf: &NetBuffer) -> DevResult {
            if self
                .daemon_file
                .write(tx_buf.packet())
                .map_err(|_| DevError::Io)?
                != tx_buf.packet().len()
            {
                Err(DevError::Io)
            } else {
                Ok(())
            }
        }

        pub fn receive(&mut self) -> DevResult<NetBufferBox<'a>> {
            if let Some(mut buf) = self.rx_buffer.take() {
                buf.set_header_len(0);
                let len = self
                    .daemon_file
                    .read(buf.raw_buf_mut())
                    .map_err(|_| DevError::Io)?;
                buf.set_packet_len(len);
                Ok(buf)
            } else {
                Err(DevError::Again)
            }
        }
    }
    pub fn current_time_nanos() -> u64 {
        libax::current_time_nanos()
    }
    pub fn yield_now() {
        libax::task::yield_now();
    }
    pub const NANOS_PER_MICROS: u64 = 1_000;
}
