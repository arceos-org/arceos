extern crate alloc;

use alloc::sync::Arc;
use axerrno::{AxError, AxResult};
use axfs::monolithic_fs::{file_io::FileExt, FileIO, FileIOType};
use axio::{Read, Seek, Write};
use axnet::{TcpSocket, UdpSocket};
use axprocess::process::current_process;
use num_enum::TryFromPrimitive;
use spinlock::SpinNoIrq;

#[derive(TryFromPrimitive)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum Domain {
    AF_UNIX = 1,
    AF_INET = 2,
}

#[derive(TryFromPrimitive)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum SocketType {
    /// Provides sequenced, reliable, two-way, connection-based byte streams.
    /// An out-of-band data transmission mechanism may be supported.
    SOCK_STREAM = 1,
    /// Supports datagrams (connectionless, unreliable messages of a fixed maximum length).
    SOCK_DGRAM = 2,
    /// Provides raw network protocol access.
    SOCK_RAW = 3,
    /// Provides a reliable datagram layer that does not guarantee ordering.
    SOCK_RDM = 4,
    /// Provides a sequenced, reliable, two-way connection-based data
    /// transmission path for datagrams of fixed maximum length;
    /// a consumer is required to read an entire packet with each input system call.
    SOCK_SEQPACKET = 5,
    /// Datagram Congestion Control Protocol socket
    SOCK_DCCP = 6,
    /// Obsolete and should not be used in new programs.
    SOCK_PACKET = 10,
    /// Set O_NONBLOCK flag on the open fd
    SOCK_NONBLOCK = 0x800,
    /// Set FD_CLOEXEC flag on the new fd
    SOCK_CLOEXEC = 0x80000,
}

/// 包装内部的不同协议 Socket
/// 类似 FileDesc，impl FileIO 后加入fd_list
pub enum Socket {
    Tcp(TcpSocket),
    Udp(UdpSocket),
}

impl Socket {
    fn new(s_type: SocketType) -> Self {
        match s_type {
            SocketType::SOCK_STREAM | SocketType::SOCK_SEQPACKET => Self::Tcp(TcpSocket::new()),
            SocketType::SOCK_DGRAM => Self::Udp(UdpSocket::new()),
            _ => unimplemented!(),
        }
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        match self {
            Self::Tcp(s) => s.read(buf),
            Self::Udp(s) => s.read(buf),
        }
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        match self {
            Self::Tcp(s) => s.write(buf),
            Self::Udp(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> AxResult {
        Err(AxError::Unsupported)
    }
}

impl Seek for Socket {
    fn seek(&mut self, _pos: axio::SeekFrom) -> AxResult<u64> {
        Err(AxError::Unsupported)
    }
}

impl FileExt for Socket {
    fn readable(&self) -> bool {
        match self {
            Self::Tcp(s) => s.poll().map_or(false, |p| p.readable),
            Self::Udp(s) => s.poll().map_or(false, |p| p.readable),
        }
    }

    fn writable(&self) -> bool {
        match self {
            Self::Tcp(s) => s.poll().map_or(false, |p| p.writable),
            Self::Udp(s) => s.poll().map_or(false, |p| p.writable),
        }
    }

    fn executable(&self) -> bool {
        false
    }
}

impl FileIO for Socket {
    fn get_type(&self) -> FileIOType {
        FileIOType::Socket
    }
}

pub fn syscall_socket(domain: usize, s_type: usize, _protocol: usize) -> isize {
    let Ok(_domain) = Domain::try_from(domain) else {
        return -1;
    };
    let Ok(s_type) = SocketType::try_from(s_type) else {
        return -1;
    };

    let socket = Socket::new(s_type);
    let curr = current_process();
    let mut inner = curr.inner.lock();

    let Ok(fd) = inner.alloc_fd() else {
        return -1;
    };

    inner.fd_manager.fd_table[fd] = Some(Arc::new(SpinNoIrq::new(socket)));

    fd as isize
}
