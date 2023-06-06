use alloc::{boxed::Box, collections::VecDeque};
use core::ops::{Deref, DerefMut};

use axerrno::{ax_err, AxError, AxResult};
use axsync::Mutex;
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, State};

use super::{SocketSetWrapper, LISTEN_QUEUE_SIZE, SOCKET_SET};
use crate::SocketAddr;

const PORT_NUM: usize = 65536;

struct ListenTableEntry {
    syn_queue: VecDeque<SocketHandle>,
}

impl ListenTableEntry {
    pub fn new() -> Self {
        Self {
            syn_queue: VecDeque::with_capacity(LISTEN_QUEUE_SIZE),
        }
    }
}

impl Drop for ListenTableEntry {
    fn drop(&mut self) {
        for &handle in &self.syn_queue {
            SOCKET_SET.remove(handle);
        }
    }
}

pub struct ListenTable {
    tcp: Box<[Mutex<Option<Box<ListenTableEntry>>>]>,
}

impl ListenTable {
    pub fn new() -> Self {
        let tcp = unsafe {
            let mut buf = Box::new_uninit_slice(PORT_NUM);
            for i in 0..PORT_NUM {
                buf[i].write(Mutex::new(None));
            }
            buf.assume_init()
        };
        Self { tcp }
    }

    pub fn can_listen(&self, port: u16) -> bool {
        self.tcp[port as usize].lock().is_none()
    }

    pub fn listen(&self, port: u16) -> AxResult {
        if port == 0 {
            return ax_err!(InvalidInput, "socket listen() failed");
        }
        let mut entry = self.tcp[port as usize].lock();
        if entry.is_none() {
            *entry = Some(Box::new(ListenTableEntry::new()));
            Ok(())
        } else {
            ax_err!(AddrInUse, "socket listen() failed")
        }
    }

    pub fn unlisten(&self, port: u16) {
        debug!("socket unlisten on {}", port);
        *self.tcp[port as usize].lock() = None;
    }

    pub fn can_accept(&self, port: u16) -> AxResult<bool> {
        if let Some(entry) = self.tcp[port as usize].lock().deref() {
            if entry.syn_queue.iter().any(|&handle| {
                let (connected, _) = get_socket_info(handle);
                connected
            }) {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            ax_err!(InvalidInput, "socket accept() failed: not listen")
        }
    }

    pub fn accept(&self, port: u16) -> AxResult<(SocketHandle, Option<SocketAddr>)> {
        if let Some(entry) = self.tcp[port as usize].lock().deref_mut() {
            let syn_queue = &mut entry.syn_queue;
            if let Some(&handle) = syn_queue.front() {
                // In most cases, the order in which sockets establish connections
                // is the same as the order in which they join the SYN queue. That
                // is, the front of the queue connects first. At this point, we can
                // use `pop_front` to speed up queue deletion.
                let (connected, peer_addr) = get_socket_info(handle);
                if connected {
                    syn_queue.pop_front();
                    return Ok((handle, peer_addr));
                }
            } else {
                return Err(AxError::WouldBlock);
            }
            if let Some((idx, peer_addr)) =
                syn_queue
                    .iter()
                    .enumerate()
                    .skip(1)
                    .find_map(|(idx, &handle)| {
                        let (connected, peer_addr) = get_socket_info(handle);
                        if connected {
                            Some((idx, peer_addr))
                        } else {
                            None
                        }
                    })
            {
                warn!(
                    "slow removal in SYN queue: index = {}, len = {}!",
                    idx,
                    syn_queue.len()
                );
                // this removal can be slow
                let handle = syn_queue.remove(idx).unwrap();
                Ok((handle, peer_addr))
            } else {
                // wait for connection
                Err(AxError::WouldBlock)
            }
        } else {
            ax_err!(InvalidInput, "socket accept() failed: not listen")
        }
    }

    pub fn incoming_tcp_packet(&self, src: SocketAddr, dst: SocketAddr) {
        if let Some(entry) = self.tcp[dst.port as usize].lock().deref_mut() {
            if entry.syn_queue.len() >= LISTEN_QUEUE_SIZE {
                // SYN queue is full, drop the packet
                warn!("SYN queue overflow!");
                return;
            }
            let mut socket = SocketSetWrapper::new_tcp_socket();
            if socket.listen(dst).is_ok() {
                let handle = SOCKET_SET.add(socket);
                debug!(
                    "socket {}: prepare for connection {} -> {}",
                    handle, src, dst
                );
                entry.syn_queue.push_back(handle);
            }
        }
    }
}

fn get_socket_info(handle: SocketHandle) -> (bool, Option<SocketAddr>) {
    let (connected, peer_addr) = SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
        (
            !matches!(socket.state(), State::Listen | State::SynReceived),
            socket.remote_endpoint(),
        )
    });
    (connected, peer_addr)
}
