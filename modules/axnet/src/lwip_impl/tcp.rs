use core::{ffi::c_void, pin::Pin};

use crate::{net_impl::driver::lwip_loop_once, SocketAddr};
use alloc::{boxed::Box, collections::VecDeque};
use axerrno::{ax_err, AxResult};
use axtask::yield_now;
use lwip_rust::bindings::{
    err_t, ip_addr_t, pbuf, pbuf_free, tcp_arg, tcp_close, tcp_connect, tcp_new, tcp_output,
    tcp_pcb, tcp_recv, tcp_recved, tcp_write,
};

use super::LWIP_MUTEX;
struct TcpSocketInner {
    remote_closed: bool,
    connect_result: i8,
    recv_queue: VecDeque<*mut pbuf>,
}
pub struct TcpSocket {
    pcb: *mut tcp_pcb,
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
        socket_inner.recv_queue.push_back(p);
    }
    0
}

impl TcpSocket {
    pub fn new() -> Self {
        debug!("[TcpSocket] new");
        let guard = LWIP_MUTEX.lock();
        let mut socket = Self {
            pcb: unsafe { tcp_new() },
            inner: Box::pin(TcpSocketInner {
                remote_closed: false,
                connect_result: 0,
                recv_queue: VecDeque::new(),
            }),
        };
        unsafe {
            tcp_arg(
                socket.pcb,
                socket.inner.as_mut().get_mut() as *mut _ as *mut c_void,
            );
            tcp_recv(socket.pcb, Some(recv_callback));
        }
        drop(guard);
        socket
    }

    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        ax_err!(Unsupported, "LWIP Unimplemented")
    }

    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        ax_err!(Unsupported, "LWIP Unimplemented")
    }

    pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
        debug!("[TcpSocket] connect to {:#?}", addr);
        let ip_addr: ip_addr_t = addr.addr.into();
        self.inner.connect_result = 1;

        // lock lwip
        let guard = LWIP_MUTEX.lock();
        unsafe {
            debug!("[TcpSocket] tcp_connect");
            match tcp_connect(self.pcb, &ip_addr, addr.port, Some(connect_callback)) {
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

    pub fn bind(&mut self, _addr: SocketAddr) -> AxResult {
        ax_err!(Unsupported, "LWIP Unimplemented")
    }

    pub fn listen(&mut self) -> AxResult {
        ax_err!(Unsupported, "LWIP Unimplemented")
    }

    pub fn accept(&mut self) -> AxResult<TcpSocket> {
        ax_err!(Unsupported, "LWIP Unimplemented")
    }

    pub fn shutdown(&self) -> AxResult {
        ax_err!(Unsupported, "LWIP Unimplemented")
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
                let p: *mut pbuf = self.inner.recv_queue.pop_front().unwrap();
                let len = unsafe { (*p).len as usize };
                let payload = unsafe { (*p).payload };
                let payload = unsafe { core::slice::from_raw_parts_mut(payload as *mut u8, len) };
                buf[0..len].copy_from_slice(payload);
                let guard = LWIP_MUTEX.lock();
                unsafe {
                    pbuf_free(p);
                    tcp_recved(self.pcb, len as u16);
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
            match tcp_write(self.pcb, buf.as_ptr() as *const _, buf.len() as u16, 0) {
                0 => {}
                _ => {
                    return ax_err!(Unsupported, "LWIP Unsupported");
                }
            }
            debug!("[TcpSocket] tcp_output");
            match tcp_output(self.pcb) {
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
        let guard = LWIP_MUTEX.lock();
        unsafe {
            tcp_arg(self.pcb, 0 as *mut c_void);
            tcp_recv(self.pcb, None);
            match tcp_close(self.pcb) {
                0 => {}
                e => {
                    error!("LWIP tcp_close failed: {}", e);
                }
            }
        }
        drop(guard);
    }
}
