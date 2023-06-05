use crate::{
    net_impl::{driver::lwip_loop_once, RECV_QUEUE_LEN},
    IpAddr, SocketAddr,
};
use alloc::{boxed::Box, collections::VecDeque};
use axerrno::{ax_err, AxError, AxResult};
use axio::PollState;
use axsync::Mutex;
use axtask::yield_now;
use core::{ffi::c_void, pin::Pin, ptr::null_mut};
use lwip_rust::bindings::{
    err_enum_t_ERR_MEM, err_enum_t_ERR_OK, err_enum_t_ERR_RTE, err_enum_t_ERR_USE,
    err_enum_t_ERR_VAL, ip_addr_t, pbuf, pbuf_alloc, pbuf_free, pbuf_layer_PBUF_TRANSPORT,
    pbuf_type_PBUF_RAM, u16_t, udp_bind, udp_connect, udp_new, udp_pcb, udp_recv, udp_remove,
    udp_sendto,
};

use super::LWIP_MUTEX;

struct UdpPcbPointer(*mut udp_pcb);
unsafe impl Send for UdpPcbPointer {}
struct PbuffPointer(*mut pbuf);
unsafe impl Send for PbuffPointer {}

struct UdpSocketInner {
    nonblock: bool,
    // (pbuf, offser, addr)
    recv_queue: Mutex<VecDeque<(PbuffPointer, usize, SocketAddr)>>,
}

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    pcb: UdpPcbPointer,
    inner: Pin<Box<UdpSocketInner>>,
}

extern "C" fn udp_recv_callback(
    arg: *mut ::core::ffi::c_void,
    _pcb: *mut udp_pcb,
    p: *mut pbuf,
    addr: *const ip_addr_t,
    port: u16_t,
) {
    let socket_inner = unsafe { &mut *(arg as *mut UdpSocketInner) };
    if p.is_null() {
        error!("[UdpSocket][udp_recv_callback] p is null");
    } else {
        debug!(
            "[UdpSocket][udp_recv_callback] p is not null, len: {}, tot_len: {}",
            unsafe { (*p).len },
            unsafe { (*p).tot_len }
        );
        socket_inner.recv_queue.lock().push_back((
            PbuffPointer(p),
            0,
            SocketAddr::new(unsafe { *addr }.into(), port),
        ));
    }
}

impl UdpSocket {
    /// Creates a new UDP socket.
    pub fn new() -> Self {
        debug!("[UdpSocket] new");
        let _guard = LWIP_MUTEX.lock();
        let mut socket = Self {
            pcb: UdpPcbPointer(unsafe { udp_new() }),
            inner: Box::pin(UdpSocketInner {
                nonblock: false,
                recv_queue: Mutex::new(VecDeque::with_capacity(RECV_QUEUE_LEN)),
            }),
        };
        unsafe {
            udp_recv(
                socket.pcb.0,
                Some(udp_recv_callback),
                socket.inner.as_mut().get_mut() as *mut _ as *mut c_void,
            );
        }
        socket
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        if self.pcb.0.is_null() {
            Err(AxError::NotConnected)
        } else {
            let _guard = LWIP_MUTEX.lock();
            let addr = unsafe { (*self.pcb.0).local_ip };
            let port = unsafe { (*self.pcb.0).local_port };
            trace!(
                "[UdpSocket] local_addr: {:#?}:{:#?}",
                IpAddr::from(addr),
                port
            );
            Ok(SocketAddr {
                addr: addr.into(),
                port,
            })
        }
    }

    /// Returns the remote address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        if self.pcb.0.is_null() {
            Err(AxError::NotConnected)
        } else {
            let _guard = LWIP_MUTEX.lock();
            let addr = unsafe { (*self.pcb.0).remote_ip };
            let port = unsafe { (*self.pcb.0).remote_port };
            trace!(
                "[UdpSocket] peer_addr: {:#?}:{:#?}",
                IpAddr::from(addr),
                port
            );
            Ok(SocketAddr {
                addr: addr.into(),
                port,
            })
        }
    }

    /// Moves this UDP socket into or out of nonblocking mode.
    ///
    /// This will result in `recv`, `recv_from`, `send`, and `send_to`
    /// operations becoming nonblocking, i.e., immediately returning from their
    /// calls. If the IO operation is successful, `Ok` is returned and no
    /// further action is required. If the IO operation could not be completed
    /// and needs to be retried, an error with kind
    /// [`Err(WouldBlock)`](AxError::WouldBlock) is returned.
    pub fn set_nonblocking(&mut self, nonblocking: bool) {
        self.inner.nonblock = nonblocking;
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`send_to`](Self::send_to) and
    /// [`recv_from`](Self::recv_from).
    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        debug!("[UdpSocket] bind to {:#?}", addr);
        let mut addr = addr;
        if addr.port == 0 {
            addr.port = get_ephemeral_port()?;
        }
        let _guard = LWIP_MUTEX.lock();
        unsafe {
            #[allow(non_upper_case_globals)]
            match udp_bind(self.pcb.0, &addr.addr.into(), addr.port) as i32 {
                err_enum_t_ERR_OK => Ok(()),
                err_enum_t_ERR_USE => {
                    ax_err!(AlreadyExists, "LWIP [udp_bind] Port already in use.")
                }
                _ => ax_err!(InvalidInput, "LWIP [udp_bind] Failed."),
            }
        }
    }

    /// Transmits data in the given buffer to the given address.
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> AxResult<usize> {
        trace!("[UdpSocket] send (len = {})", buf.len());
        let copy_len = core::cmp::min(buf.len(), 1472);
        unsafe {
            let _guard = LWIP_MUTEX.lock();
            let p = pbuf_alloc(
                pbuf_layer_PBUF_TRANSPORT,
                copy_len as u16,
                pbuf_type_PBUF_RAM,
            );
            if p.is_null() {
                return ax_err!(NoMemory, "LWIP Out of memory.");
            }
            let payload = (*p).payload;
            let payload = core::slice::from_raw_parts_mut(payload as *mut u8, copy_len);
            payload.copy_from_slice(buf);
            (*p).len = copy_len as u16;
            (*p).tot_len = copy_len as u16;

            trace!("[UdpSocket] udp_sendto");

            #[allow(non_upper_case_globals)]
            match udp_sendto(self.pcb.0, p, &addr.addr.into(), addr.port) as i32 {
                err_enum_t_ERR_OK => {}
                err_enum_t_ERR_MEM => return ax_err!(NoMemory, "LWIP Out of memory."),
                err_enum_t_ERR_RTE => {
                    return ax_err!(
                        BadState,
                        "LWIP Could not find route to destination address."
                    )
                }
                err_enum_t_ERR_VAL => {
                    return ax_err!(InvalidInput, "LWIP No PCB or PCB is dual-stack.")
                }
                _ => return ax_err!(InvalidInput, "LWIP Invalid input."),
            }
        }
        lwip_loop_once();
        Ok(copy_len)
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recv_from(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        trace!("[UdpSocket] recvfrom");
        loop {
            lwip_loop_once();
            let mut recv_queue = self.inner.recv_queue.lock();
            let res = if recv_queue.len() == 0 {
                Err(AxError::WouldBlock)
            } else {
                let (p, offset, addr) = recv_queue.pop_front().unwrap();
                let p: *mut pbuf = p.0;
                let len = unsafe { (*p).len as usize };
                let tot_len = unsafe { (*p).tot_len as usize };
                if len != tot_len {
                    // TODO: pbuf chain
                    error!("[TcpSocket] recv pbuf len != tot_len");
                    return ax_err!(Unsupported, "LWIP [recv] pbuf len != tot_len");
                }
                let payload = unsafe { (*p).payload };
                let payload = unsafe { core::slice::from_raw_parts_mut(payload as *mut u8, len) };

                let copy_len = core::cmp::min(len - offset, buf.len());
                buf[0..copy_len].copy_from_slice(&payload[offset..offset + copy_len]);
                if offset + copy_len < len {
                    recv_queue.push_front((PbuffPointer(p), offset + copy_len, addr));
                } else {
                    let guard = LWIP_MUTEX.lock();
                    unsafe {
                        pbuf_free(p);
                    }
                    drop(guard);
                }

                Ok((copy_len, addr))
            };
            drop(recv_queue);
            match res {
                Ok((len, addr)) => {
                    trace!("[UdpSocket] recv done (len: {}): {:?}", len, &buf[0..len]);
                    return Ok((len, addr));
                }
                Err(AxError::WouldBlock) => {
                    if self.inner.nonblock {
                        return Err(AxError::WouldBlock);
                    } else {
                        yield_now();
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
    }

    /// Connects to the given address and port.
    ///
    /// The local port will be generated automatically if the socket is not bound.
    /// It's must be called before [`send`](Self::send) and
    /// [`recv`](Self::recv).
    pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
        debug!("[UdpSocket] connect to {:#?}", addr);
        let ip_addr: ip_addr_t = addr.addr.into();
        let _guard = LWIP_MUTEX.lock();
        unsafe {
            #[allow(non_upper_case_globals)]
            match udp_connect(self.pcb.0, &ip_addr, addr.port) as i32 {
                err_enum_t_ERR_OK => Ok(()),
                _ => ax_err!(InvalidInput, "LWIP [udp_connect] Failed."),
            }
        }
    }

    /// Transmits data in the given buffer to the remote address to which it is connected.
    pub fn send(&self, _buf: &[u8]) -> AxResult<usize> {
        ax_err!(Unsupported, "LWIP Unsupported UDP send")
    }

    /// Recv data in the given buffer from the remote address to which it is connected.
    pub fn recv(&self, _buf: &mut [u8]) -> AxResult<usize> {
        ax_err!(Unsupported, "LWIP Unsupported UDP recv")
    }

    /// Close the socket.
    pub fn shutdown(&mut self) -> AxResult {
        if !self.pcb.0.is_null() {
            let _guard = LWIP_MUTEX.lock();
            unsafe {
                udp_recv(self.pcb.0, None, null_mut());
                udp_remove(self.pcb.0);
            }
            self.pcb.0 = null_mut();
            lwip_loop_once();
            Ok(())
        } else {
            ax_err!(InvalidInput)
        }
    }

    /// Receives data from the socket, stores it in the given buffer, without removing it from the queue.
    pub fn peek_from(&self, _buf: &mut [u8]) -> AxResult<(usize, SocketAddr)> {
        ax_err!(Unsupported, "LWIP Unsupported UDP peek_from")
    }

    /// Detect whether the socket needs to receive/can send.
    ///
    /// Return is <need to receive, can send>
    pub fn poll(&self) -> AxResult<PollState> {
        lwip_loop_once();
        Ok(PollState {
            readable: self.inner.recv_queue.lock().len() != 0,
            writable: true,
        })
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        debug!("[UdpSocket] drop");
        self.shutdown().unwrap();
    }
}

impl Default for UdpSocket {
    fn default() -> Self {
        Self::new()
    }
}

fn get_ephemeral_port() -> AxResult<u16> {
    const PORT_START: u16 = 0xc000;
    const PORT_END: u16 = 0xffff;
    static CURR: Mutex<u16> = Mutex::new(PORT_START);
    let mut curr = CURR.lock();

    let port = *curr;
    if *curr == PORT_END {
        *curr = PORT_START;
    } else {
        *curr += 1;
    }
    Ok(port)
}
