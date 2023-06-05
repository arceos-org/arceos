use crate::{
    net_impl::{driver::lwip_loop_once, ACCEPT_QUEUE_LEN, RECV_QUEUE_LEN},
    IpAddr, SocketAddr,
};
use alloc::{boxed::Box, collections::VecDeque};
use axerrno::{ax_err, AxError, AxResult};
use axio::PollState;
use axsync::Mutex;
use axtask::yield_now;
use core::{ffi::c_void, pin::Pin, ptr::null_mut};
use lwip_rust::bindings::{
    err_enum_t_ERR_MEM, err_enum_t_ERR_OK, err_enum_t_ERR_USE, err_enum_t_ERR_VAL, err_t,
    ip_addr_t, pbuf, pbuf_free, tcp_accept, tcp_arg, tcp_bind, tcp_close, tcp_connect,
    tcp_listen_with_backlog, tcp_new, tcp_output, tcp_pcb, tcp_recv, tcp_recved, tcp_state_LISTEN,
    tcp_write, TCP_DEFAULT_LISTEN_BACKLOG, TCP_MSS,
};

use super::LWIP_MUTEX;

struct TcpPcbPointer(*mut tcp_pcb);
unsafe impl Send for TcpPcbPointer {}
struct PbuffPointer(*mut pbuf);
unsafe impl Send for PbuffPointer {}

struct TcpSocketInner {
    nonblock: bool,
    remote_closed: bool,
    connect_result: i8,
    // (pcb, offset)
    recv_queue: Mutex<VecDeque<(PbuffPointer, usize)>>,
    accept_queue: Mutex<VecDeque<TcpSocket>>,
}

/// A TCP socket that provides POSIX-like APIs.
pub struct TcpSocket {
    pcb: TcpPcbPointer,
    inner: Pin<Box<TcpSocketInner>>,
}

extern "C" fn connect_callback(arg: *mut c_void, _tpcb: *mut tcp_pcb, err: err_t) -> err_t {
    debug!("[TcpSocket] connect_callback: {:#?}", err);
    let socket_inner = unsafe { &mut *(arg as *mut TcpSocketInner) };
    socket_inner.connect_result = err;
    err
}

extern "C" fn recv_callback(
    arg: *mut c_void,
    _tpcb: *mut tcp_pcb,
    p: *mut pbuf,
    err: err_t,
) -> err_t {
    debug!("[TcpSocket] recv_callback: {:#?}", err);
    if err != 0 {
        error!("[TcpSocket][recv_callback] err: {:#?}", err);
        return err;
    }
    let socket_inner = unsafe { &mut *(arg as *mut TcpSocketInner) };
    if p.is_null() {
        debug!("[TcpSocket][recv_callback] p is null, remote close");
        socket_inner.remote_closed = true;
    } else {
        debug!(
            "[TcpSocket][recv_callback] p is not null, len: {}, tot_len: {}",
            unsafe { (*p).len },
            unsafe { (*p).tot_len }
        );
        socket_inner
            .recv_queue
            .lock()
            .push_back((PbuffPointer(p), 0));
        debug!(
            "[TcpSocket][recv_callback] recv_queue len: {}",
            socket_inner.recv_queue.lock().len()
        );
    }
    0
}

extern "C" fn accept_callback(arg: *mut c_void, newpcb: *mut tcp_pcb, err: err_t) -> err_t {
    debug!("[TcpSocket] accept_callback: {:#?}", err);
    if err != 0 {
        debug!("[TcpSocket][accept_callback] err: {:#?}", err);
        return err;
    }
    let socket_inner = unsafe { &mut *(arg as *mut TcpSocketInner) };
    let mut socket = TcpSocket {
        pcb: TcpPcbPointer(newpcb),
        inner: Box::pin(TcpSocketInner {
            nonblock: false,
            remote_closed: false,
            connect_result: 0,
            recv_queue: Mutex::new(VecDeque::with_capacity(RECV_QUEUE_LEN)),
            accept_queue: Mutex::new(VecDeque::new()),
        }),
    };
    unsafe {
        tcp_arg(
            socket.pcb.0,
            socket.inner.as_mut().get_mut() as *mut _ as *mut c_void,
        );
        tcp_recv(socket.pcb.0, Some(recv_callback));
    }
    socket_inner.accept_queue.lock().push_back(socket);
    debug!(
        "[TcpSocket][accept_callback] accept_queue len: {}",
        socket_inner.accept_queue.lock().len()
    );
    0
}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub fn new() -> Self {
        debug!("[TcpSocket] new");
        let guard = LWIP_MUTEX.lock();
        let mut socket = Self {
            pcb: TcpPcbPointer(unsafe { tcp_new() }),
            inner: Box::pin(TcpSocketInner {
                nonblock: false,
                remote_closed: false,
                connect_result: 0,
                recv_queue: Mutex::new(VecDeque::new()),
                accept_queue: Mutex::new(VecDeque::with_capacity(ACCEPT_QUEUE_LEN)),
            }),
        };
        unsafe {
            tcp_arg(
                socket.pcb.0,
                socket.inner.as_mut().get_mut() as *mut _ as *mut c_void,
            );
        }
        drop(guard);
        socket
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        if self.pcb.0.is_null() {
            Err(AxError::NotConnected)
        } else {
            let guard = LWIP_MUTEX.lock();
            let addr = unsafe { (*self.pcb.0).local_ip };
            let port = unsafe { (*self.pcb.0).local_port };
            drop(guard);
            trace!(
                "[TcpSocket] local_addr: {:#?}:{:#?}",
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
            let guard = LWIP_MUTEX.lock();
            let addr = unsafe { (*self.pcb.0).remote_ip };
            let port = unsafe { (*self.pcb.0).remote_port };
            drop(guard);
            trace!(
                "[TcpSocket] peer_addr: {:#?}:{:#?}",
                IpAddr::from(addr),
                port
            );
            Ok(SocketAddr {
                addr: addr.into(),
                port,
            })
        }
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind  [`Err(WouldBlock)`](AxError::WouldBlock) is
    /// returned.
    pub fn set_nonblocking(&mut self, nonblocking: bool) {
        self.inner.nonblock = nonblocking;
    }

    /// Connects to the given address and port.
    ///
    /// The local port is generated automatically.
    pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
        debug!("[TcpSocket] connect to {:#?}", addr);
        let ip_addr: ip_addr_t = addr.addr.into();
        self.inner.connect_result = 1;

        // lock lwip
        let guard = LWIP_MUTEX.lock();
        unsafe {
            debug!("[TcpSocket] set recv_callback");
            tcp_recv(self.pcb.0, Some(recv_callback));

            debug!("[TcpSocket] tcp_connect");
            #[allow(non_upper_case_globals)]
            match tcp_connect(self.pcb.0, &ip_addr, addr.port, Some(connect_callback)) as i32 {
                err_enum_t_ERR_OK => {}
                err_enum_t_ERR_VAL => {
                    return ax_err!(InvalidInput, "LWIP [tcp_connect] Invalid input.");
                }
                _ => {
                    return ax_err!(Unsupported, "LWIP [tcp_connect] Failed.");
                }
            };
        }
        drop(guard);

        // wait for connect
        debug!("[TcpSocket] wait for connect");
        lwip_loop_once();
        #[allow(clippy::while_immutable_condition)]
        while self.inner.connect_result == 1 {
            yield_now();
            lwip_loop_once();
        }
        debug!("[TcpSocket] connect result: {}", self.inner.connect_result);

        if self.inner.connect_result == 0 {
            Ok(())
        } else {
            ax_err!(Unsupported, "LWIP [connect_result] Unsupported")
        }
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// If the given port is 0, it generates one automatically.
    ///
    /// It's must be called before [`listen`](Self::listen) and
    /// [`accept`](Self::accept).
    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        debug!("[TcpSocket] bind to {:#?}", addr);
        let guard = LWIP_MUTEX.lock();
        unsafe {
            #[allow(non_upper_case_globals)]
            match tcp_bind(self.pcb.0, &addr.addr.into(), addr.port) as i32 {
                err_enum_t_ERR_OK => {}
                err_enum_t_ERR_USE => {
                    return ax_err!(AddrInUse, "LWIP [tcp_bind] Port already in use.");
                }
                err_enum_t_ERR_VAL => {
                    return ax_err!(
                        InvalidInput,
                        "LWIP [tcp_bind] The PCB is not in a valid state."
                    );
                }
                _ => {
                    return ax_err!(Unsupported, "LWIP [tcp_bind] Failed.");
                }
            };
        }
        drop(guard);
        Ok(())
    }

    /// Starts listening on the bound address and port.
    ///
    /// It's must be called after [`bind`](Self::bind) and before
    /// [`accept`](Self::accept).
    pub fn listen(&mut self) -> AxResult {
        debug!("[TcpSocket] listen");
        let guard = LWIP_MUTEX.lock();
        unsafe {
            self.pcb.0 = tcp_listen_with_backlog(self.pcb.0, TCP_DEFAULT_LISTEN_BACKLOG as u8);
            tcp_arg(
                self.pcb.0,
                self.inner.as_mut().get_mut() as *mut _ as *mut c_void,
            );
            tcp_accept(self.pcb.0, Some(accept_callback));
        }
        drop(guard);
        // TODO: check if listen failed
        Ok(())
    }

    /// Accepts a new connection.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, a new [`TcpSocket`] is returned.
    ///
    /// It's must be called after [`bind`](Self::bind) and [`listen`](Self::listen).
    pub fn accept(&self) -> AxResult<TcpSocket> {
        debug!("[TcpSocket] accept");
        loop {
            lwip_loop_once();
            let mut accept_queue = self.inner.accept_queue.lock();
            if accept_queue.len() != 0 {
                return Ok(accept_queue.pop_front().unwrap());
            }
            drop(accept_queue);
            if self.inner.nonblock {
                return Err(AxError::WouldBlock);
            } else {
                yield_now();
            }
        }
    }

    /// Close the connection.
    pub fn shutdown(&mut self) -> AxResult {
        if !self.pcb.0.is_null() {
            unsafe {
                let _guard = LWIP_MUTEX.lock();
                tcp_arg(self.pcb.0, null_mut());
                if (*self.pcb.0).state == tcp_state_LISTEN {
                    tcp_accept(self.pcb.0, None);
                } else {
                    tcp_recv(self.pcb.0, None);
                }

                warn!("[TcpSocket] tcp_close");
                #[allow(non_upper_case_globals)]
                match tcp_close(self.pcb.0) as i32 {
                    err_enum_t_ERR_OK => {}
                    e => {
                        error!("LWIP tcp_close failed: {}", e);
                        return ax_err!(Unsupported, "LWIP [tcp_close] failed");
                    }
                }
            }
            self.pcb.0 = null_mut();
            lwip_loop_once();
            Ok(())
        } else {
            Err(AxError::NotConnected)
        }
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recv(&self, buf: &mut [u8]) -> AxResult<usize> {
        trace!("[TcpSocket] recv");
        loop {
            if self.inner.remote_closed {
                return Ok(0);
            }
            lwip_loop_once();
            let mut recv_queue = self.inner.recv_queue.lock();
            let res = if recv_queue.len() == 0 {
                Ok(0)
            } else {
                let (p, offset) = recv_queue.pop_front().unwrap();
                let p = p.0;
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
                    recv_queue.push_front((PbuffPointer(p), offset + copy_len));
                } else {
                    let guard = LWIP_MUTEX.lock();
                    unsafe {
                        pbuf_free(p);
                        tcp_recved(self.pcb.0, len as u16);
                    }
                    drop(guard);
                }

                Ok(copy_len)
            };
            drop(recv_queue);
            match res {
                Ok(0) => {
                    if self.inner.nonblock {
                        return Err(AxError::WouldBlock);
                    } else {
                        yield_now();
                    }
                }
                Ok(len) => {
                    trace!("[TcpSocket] recv done (len: {}): {:?}", len, &buf[0..len]);
                    return Ok(len);
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
    }

    /// Transmits data in the given buffer.
    pub fn send(&self, buf: &[u8]) -> AxResult<usize> {
        trace!("[TcpSocket] send (len = {})", buf.len());
        let copy_len = core::cmp::min(buf.len(), TCP_MSS as usize);
        unsafe {
            let _guard = LWIP_MUTEX.lock();
            trace!("[TcpSocket] tcp_write");
            #[allow(non_upper_case_globals)]
            match tcp_write(self.pcb.0, buf.as_ptr() as *const _, copy_len as u16, 0) as i32 {
                err_enum_t_ERR_OK => {}
                err_enum_t_ERR_MEM => {
                    return ax_err!(NoMemory, "LWIP [tcp_write] Out of memory.");
                }
                _ => {
                    return ax_err!(Unsupported, "LWIP [tcp_write] Failed.");
                }
            }
            trace!("[TcpSocket] tcp_output");
            #[allow(non_upper_case_globals)]
            match tcp_output(self.pcb.0) as i32 {
                err_enum_t_ERR_OK => {}
                _ => {
                    return ax_err!(Unsupported, "LWIP [tcp_output] Failed.");
                }
            }
        };
        lwip_loop_once();
        trace!("[TcpSocket] send done (len: {})", copy_len);
        Ok(copy_len)
    }

    /// Detect whether the socket needs to receive/can send.
    ///
    /// Return is <need to receive, can send>
    pub fn poll(&self) -> AxResult<PollState> {
        trace!("poll pcbstate: {:?}", unsafe { (*self.pcb.0).state });
        lwip_loop_once();
        if unsafe { (*self.pcb.0).state } == tcp_state_LISTEN {
            // listener
            Ok(PollState {
                readable: self.inner.accept_queue.lock().len() != 0,
                writable: false,
            })
        } else {
            // stream
            Ok(PollState {
                readable: self.inner.recv_queue.lock().len() != 0,
                writable: true,
            })
        }
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        debug!("[TcpSocket] drop");
        self.shutdown().unwrap();
    }
}

impl Default for TcpSocket {
    fn default() -> Self {
        Self::new()
    }
}
