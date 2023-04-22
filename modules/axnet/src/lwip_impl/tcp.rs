use crate::{net_impl::driver::lwip_loop_once, IpAddr, SocketAddr};
use alloc::{boxed::Box, collections::VecDeque};
use axerrno::{ax_err, AxResult};
use axtask::yield_now;
use core::{ffi::c_void, pin::Pin};
use lwip_rust::bindings::{
    err_t, ip_addr_t, pbuf, pbuf_free, tcp_accept, tcp_arg, tcp_bind, tcp_close, tcp_connect,
    tcp_listen_with_backlog, tcp_new, tcp_output, tcp_pcb, tcp_recv, tcp_recved, tcp_write,
    TCP_DEFAULT_LISTEN_BACKLOG,
};

use super::LWIP_MUTEX;

struct TcpPcbPointer(*mut tcp_pcb);
unsafe impl Send for TcpPcbPointer {}
struct PbuffPointer(*mut pbuf);
unsafe impl Send for PbuffPointer {}

struct TcpSocketInner {
    remote_closed: bool,
    connect_result: i8,
    recv_queue: VecDeque<PbuffPointer>,
    accept_queue: VecDeque<TcpSocket>,
}
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
    let socket_inner = unsafe { &mut *(arg as *mut TcpSocketInner) };
    if err != 0 {
        return err;
    }
    if p.is_null() {
        debug!("[TcpSocket][recv_callback] p is null, remote close");
        socket_inner.remote_closed = true;
    } else {
        debug!(
            "[TcpSocket][recv_callback] p is not null, len: {}, tot_len: {}",
            unsafe { (*p).len },
            unsafe { (*p).tot_len }
        );
        socket_inner.recv_queue.push_back(PbuffPointer(p));
    }
    0
}

extern "C" fn accept_callback(arg: *mut c_void, newpcb: *mut tcp_pcb, err: err_t) -> err_t {
    debug!("[TcpSocket] accept_callback: {:#?}", err);
    let socket_inner = unsafe { &mut *(arg as *mut TcpSocketInner) };
    if err != 0 {
        return err;
    }
    let mut socket = TcpSocket {
        pcb: TcpPcbPointer(newpcb),
        inner: Box::pin(TcpSocketInner {
            remote_closed: false,
            connect_result: 0,
            recv_queue: VecDeque::new(),
            accept_queue: VecDeque::new(),
        }),
    };
    unsafe {
        tcp_arg(
            socket.pcb.0,
            socket.inner.as_mut().get_mut() as *mut _ as *mut c_void,
        );
        tcp_recv(socket.pcb.0, Some(recv_callback));
    }
    socket_inner.accept_queue.push_back(socket);
    0
}

impl TcpSocket {
    pub fn new() -> Self {
        debug!("[TcpSocket] new");
        let guard = LWIP_MUTEX.lock();
        let mut socket = Self {
            pcb: TcpPcbPointer(unsafe { tcp_new() }),
            inner: Box::pin(TcpSocketInner {
                remote_closed: false,
                connect_result: 0,
                recv_queue: VecDeque::new(),
                accept_queue: VecDeque::new(),
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

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        if self.pcb.0.is_null() {
            return ax_err!(NotConnected);
        } else {
            let guard = LWIP_MUTEX.lock();
            let addr = unsafe { (*self.pcb.0).local_ip };
            let port = unsafe { (*self.pcb.0).local_port };
            drop(guard);
            debug!(
                "[TcpSocket] local_addr: {:#?}:{:#?}",
                IpAddr::from(addr),
                port
            );
            return Ok(SocketAddr {
                addr: addr.into(),
                port,
            });
        }
    }

    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        if self.pcb.0.is_null() {
            return ax_err!(NotConnected);
        } else {
            let guard = LWIP_MUTEX.lock();
            let addr = unsafe { (*self.pcb.0).remote_ip };
            let port = unsafe { (*self.pcb.0).remote_port };
            drop(guard);
            debug!(
                "[TcpSocket] peer_addr: {:#?}:{:#?}",
                IpAddr::from(addr),
                port
            );
            return Ok(SocketAddr {
                addr: addr.into(),
                port,
            });
        }
    }

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
            match tcp_connect(self.pcb.0, &ip_addr, addr.port, Some(connect_callback)) {
                0 => {}
                _ => {
                    return ax_err!(Unsupported, "LWIP Unsupported");
                }
            };
        }
        drop(guard);

        // wait for connect
        debug!("[TcpSocket] wait for connect");
        lwip_loop_once();
        while self.inner.connect_result == 1 {
            yield_now();
            lwip_loop_once();
        }
        debug!("[TcpSocket] connect result: {}", self.inner.connect_result);

        if self.inner.connect_result == 0 {
            Ok(())
        } else {
            ax_err!(Unsupported, "LWIP Unsupported")
        }
    }

    pub fn bind(&mut self, addr: SocketAddr) -> AxResult {
        debug!("[TcpSocket] bind to {:#?}", addr);
        // TODO: check if already bound
        let guard = LWIP_MUTEX.lock();
        unsafe {
            tcp_bind(self.pcb.0, &addr.addr.into(), addr.port);
        }
        drop(guard);
        Ok(())
    }

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

    pub fn accept(&mut self) -> AxResult<TcpSocket> {
        debug!("[TcpSocket] accept");
        if self.inner.accept_queue.len() != 0 {
            return Ok(self.inner.accept_queue.pop_front().unwrap());
        }
        loop {
            lwip_loop_once();
            if self.inner.accept_queue.len() != 0 {
                return Ok(self.inner.accept_queue.pop_front().unwrap());
            }
            yield_now();
        }
    }

    pub fn shutdown(&mut self) -> AxResult {
        if !self.pcb.0.is_null() {
            let guard = LWIP_MUTEX.lock();
            unsafe {
                tcp_arg(self.pcb.0, 0 as *mut c_void);
                tcp_recv(self.pcb.0, None);
                tcp_accept(self.pcb.0, None);
                match tcp_close(self.pcb.0) {
                    0 => {}
                    e => {
                        error!("LWIP tcp_close failed: {}", e);
                        return ax_err!(Unsupported, "LWIP tcp_close failed");
                    }
                }
            }
            drop(guard);
            self.pcb.0 = 0 as *mut tcp_pcb;
            Ok(())
        } else {
            ax_err!(NotConnected)
        }
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        debug!("[TcpSocket] recv");
        if self.inner.remote_closed {
            return Ok(0);
        }
        loop {
            lwip_loop_once();
            match if self.inner.recv_queue.len() == 0 {
                Ok(0)
            } else {
                // TODO: len > buf.len()
                // TODO: pbuf chain
                let p: *mut pbuf = self.inner.recv_queue.pop_front().unwrap().0;
                let len = unsafe { (*p).len as usize };
                let payload = unsafe { (*p).payload };
                let payload = unsafe { core::slice::from_raw_parts_mut(payload as *mut u8, len) };
                buf[0..len].copy_from_slice(payload);
                let guard = LWIP_MUTEX.lock();
                unsafe {
                    pbuf_free(p);
                    tcp_recved(self.pcb.0, len as u16);
                }
                drop(guard);
                Ok(len)
            } {
                Ok(0) => {
                    yield_now();
                }
                Ok(len) => {
                    trace!("[TcpSocket] recv done: {:?}", &buf[0..len]);
                    return Ok(len);
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
    }

    pub fn send(&self, buf: &[u8]) -> AxResult<usize> {
        debug!("[TcpSocket] send: {:?}", buf);
        let guard = LWIP_MUTEX.lock();
        unsafe {
            debug!("[TcpSocket] tcp_write");
            match tcp_write(self.pcb.0, buf.as_ptr() as *const _, buf.len() as u16, 0) {
                0 => {}
                _ => {
                    return ax_err!(Unsupported, "LWIP Unsupported");
                }
            }
            debug!("[TcpSocket] tcp_output");
            match tcp_output(self.pcb.0) {
                0 => {}
                _ => {
                    return ax_err!(Unsupported, "LWIP Unsupported");
                }
            }
        }
        drop(guard);
        debug!("[TcpSocket] send done");
        Ok(buf.len())
    }
}

impl Drop for TcpSocket {
    fn drop(&mut self) {
        debug!("[TcpSocket] drop");
        self.shutdown().unwrap();
    }
}
