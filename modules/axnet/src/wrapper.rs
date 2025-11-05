use alloc::vec;

use axerrno::{AxError, AxResult};
use axsync::Mutex;
use event_listener::Event;
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::{AnySocket, Socket},
    wire::IpAddress,
};

pub(crate) struct SocketSetWrapper<'a> {
    pub inner: Mutex<SocketSet<'a>>,
    pub new_socket: Event,
}

impl<'a> SocketSetWrapper<'a> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SocketSet::new(vec![])),
            new_socket: Event::new(),
        }
    }

    pub fn add<T: AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.inner.lock().add(socket);
        debug!("socket {}: created", handle);
        self.new_socket.notify(1);
        handle
    }

    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.inner.lock();
        let socket = set.get(handle);
        f(socket)
    }

    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.inner.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }

    pub fn bind_check(&self, addr: IpAddress, port: u16) -> AxResult {
        if port == 0 {
            return Ok(());
        }

        // TODO(mivik): optimize
        let mut sockets = self.inner.lock();
        for (_, socket) in sockets.iter_mut() {
            match socket {
                Socket::Tcp(s) => {
                    let local_addr = s.get_bound_endpoint();
                    if local_addr.addr == Some(addr) && local_addr.port == port {
                        return Err(AxError::AddrInUse);
                    }
                }
                Socket::Udp(s) => {
                    if s.endpoint().addr == Some(addr) && s.endpoint().port == port {
                        return Err(AxError::AddrInUse);
                    }
                }
                _ => continue,
            };
        }
        Ok(())
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.inner.lock().remove(handle);
        debug!("socket {}: destroyed", handle);
    }
}
