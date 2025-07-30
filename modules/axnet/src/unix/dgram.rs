use alloc::{boxed::Box, vec::Vec};

use async_trait::async_trait;
use axerrno::{LinuxError, LinuxResult};
use axio::{
    PollState,
    buf::{Buf, BufMut},
};
use axsync::Mutex;
use axtask::future::block_on_interruptible;
use spin::RwLock;

use crate::{
    CMsgData, RecvFlags, RecvOptions, SendOptions, SocketAddrEx,
    options::{Configurable, GetSocketOption, SetSocketOption},
    unix::{Transport, UnixSocketAddr, with_slot},
};

const UDP_MAX_PAYLOAD_SIZE: usize = 65507;

struct Packet {
    data: Vec<u8>,
    cmsg: Vec<CMsgData>,
    sender: UnixSocketAddr,
}

struct Channel {
    data_tx: async_channel::Sender<Packet>,
}

pub struct Bind {
    data_tx: async_channel::Sender<Packet>,
}
impl Bind {
    fn connect(&self) -> Channel {
        let tx = self.data_tx.clone();
        Channel { data_tx: tx }
    }
}

pub struct DgramTransport {
    data_rx: Mutex<Option<async_channel::Receiver<Packet>>>,
    connected: RwLock<Option<Channel>>,
    local_addr: RwLock<UnixSocketAddr>,
}
impl DgramTransport {
    pub fn new() -> Self {
        DgramTransport {
            data_rx: Mutex::new(None),
            connected: RwLock::new(None),
            local_addr: RwLock::new(UnixSocketAddr::Unnamed),
        }
    }
}

impl Configurable for DgramTransport {
    fn get_option_inner(&self, _opt: &mut GetSocketOption) -> LinuxResult<bool> {
        Ok(false)
    }

    fn set_option_inner(&self, _opt: SetSocketOption) -> LinuxResult<bool> {
        Ok(false)
    }
}
#[async_trait]
impl Transport for DgramTransport {
    fn bind(&self, slot: &super::BindSlot, local_addr: &UnixSocketAddr) -> LinuxResult<()> {
        let mut slot = slot.dgram.lock();
        if slot.is_some() {
            return Err(LinuxError::EADDRINUSE);
        }
        let mut guard = self.data_rx.lock();
        if guard.is_some() {
            return Err(LinuxError::EINVAL);
        }
        let (tx, rx) = async_channel::unbounded();
        *slot = Some(Bind { data_tx: tx });
        *guard = Some(rx);
        self.local_addr.write().clone_from(local_addr);
        Ok(())
    }

    fn connect(&self, slot: &super::BindSlot, _local_addr: &UnixSocketAddr) -> LinuxResult<()> {
        let mut guard = self.connected.write();
        if guard.is_some() {
            return Err(LinuxError::EISCONN);
        }
        *guard = Some(
            slot.dgram
                .lock()
                .as_ref()
                .ok_or(LinuxError::ENOTCONN)?
                .connect(),
        );
        Ok(())
    }

    async fn accept(&self) -> LinuxResult<(Box<dyn super::Transport>, UnixSocketAddr)> {
        Err(LinuxError::EINVAL)
    }

    fn send(&self, src: &mut dyn Buf, options: SendOptions) -> LinuxResult<usize> {
        let mut message = Vec::new();
        loop {
            let chunk = src.chunk();
            let len = chunk.len().min(UDP_MAX_PAYLOAD_SIZE - message.len());
            if len == 0 {
                break;
            }

            message.extend_from_slice(&chunk[..len]);
            src.advance(len);
        }
        let len = message.len();
        let packet = Packet {
            data: message,
            cmsg: options.cmsg,
            sender: self.local_addr.read().clone(),
        };

        let connected = self.connected.read();
        if let Some(addr) = options.to {
            let addr = addr.into_unix()?;
            with_slot(&addr, |slot| {
                if let Some(bind) = slot.dgram.lock().as_ref() {
                    bind.data_tx
                        .try_send(packet)
                        .map_err(|_| LinuxError::EPIPE)?;
                    Ok(())
                } else {
                    Err(LinuxError::ENOTCONN)
                }
            })?;
        } else if let Some(chan) = connected.as_ref() {
            chan.data_tx
                .try_send(packet)
                .map_err(|_| LinuxError::EPIPE)?;
        } else {
            return Err(LinuxError::ENOTCONN);
        }
        Ok(len)
    }

    fn recv(&self, dst: &mut dyn BufMut, options: RecvOptions) -> LinuxResult<usize> {
        let mut guard = self.data_rx.lock();
        let Some(rx) = guard.as_mut() else {
            return Err(LinuxError::ENOTCONN);
        };

        block_on_interruptible(async {
            let Ok(Packet { data, cmsg, sender }) = rx.recv().await else {
                return Ok(0);
            };
            let mut read = 0;
            loop {
                let chunk = dst.chunk_mut();
                let len = chunk.len().min(data.len() - read);
                if len == 0 {
                    break;
                }

                chunk[..len].copy_from_slice(&data[read..read + len]);
                read += len;
            }
            if read < data.len() {
                warn!("UDP message truncated: {} -> {} bytes", data.len(), read);
            }

            if let Some(from) = options.from {
                *from = SocketAddrEx::Unix(sender);
            }
            if let Some(dst) = options.cmsg {
                dst.extend(cmsg);
            }

            Ok(if options.flags.contains(RecvFlags::TRUNCATE) {
                data.len()
            } else {
                read
            })
        })
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn make_pair() -> LinuxResult<(Self, Self)> {
        let (tx1, rx1) = async_channel::unbounded();
        let (tx2, rx2) = async_channel::unbounded();
        let transport1 = DgramTransport {
            data_rx: Mutex::new(Some(rx1)),
            connected: RwLock::new(Some(Channel { data_tx: tx2 })),
            local_addr: RwLock::new(UnixSocketAddr::Unnamed),
        };
        let transport2 = DgramTransport {
            data_rx: Mutex::new(Some(rx2)),
            connected: RwLock::new(Some(Channel { data_tx: tx1 })),
            local_addr: RwLock::new(UnixSocketAddr::Unnamed),
        };
        Ok((transport1, transport2))
    }
}
