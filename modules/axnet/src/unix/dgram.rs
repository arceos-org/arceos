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
    RecvFlags, SendFlags,
    options::{Configurable, GetSocketOption, SetSocketOption},
    unix::{Transport, UnixSocketAddr, with_slot},
};

const UDP_MAX_PAYLOAD_SIZE: usize = 65507;

struct Channel {
    data_tx: async_channel::Sender<Vec<u8>>,
}

pub struct Bind {
    data_tx: async_channel::Sender<Vec<u8>>,
}
impl Bind {
    fn connect(&self) -> Channel {
        let tx = self.data_tx.clone();
        Channel { data_tx: tx }
    }
}

pub struct DgramTransport {
    data_rx: Mutex<Option<async_channel::Receiver<Vec<u8>>>>,
    connected: RwLock<Option<Channel>>,
}
impl DgramTransport {
    pub fn new() -> Self {
        DgramTransport {
            data_rx: Mutex::new(None),
            connected: RwLock::new(None),
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
    fn bind(&self, slot: &super::BindSlot) -> LinuxResult<()> {
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

    fn send(
        &self,
        src: &mut dyn Buf,
        to: Option<UnixSocketAddr>,
        _flags: SendFlags,
    ) -> LinuxResult<usize> {
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

        let connected = self.connected.read();
        if let Some(addr) = to {
            with_slot(&addr, |slot| {
                if let Some(bind) = slot.dgram.lock().as_ref() {
                    bind.data_tx
                        .try_send(message.clone())
                        .expect("unbound channel cant fail");
                    Ok(())
                } else {
                    Err(LinuxError::ENOTCONN)
                }
            })?;
        } else if let Some(chan) = connected.as_ref() {
            chan.data_tx
                .try_send(message)
                .expect("unbound channel cant fail");
        } else {
            return Err(LinuxError::ENOTCONN);
        }
        Ok(len)
    }

    fn recv(&self, dst: &mut dyn BufMut, flags: RecvFlags) -> LinuxResult<usize> {
        let mut guard = self.data_rx.lock();
        let Some(rx) = guard.as_mut() else {
            return Err(LinuxError::ENOTCONN);
        };

        block_on_interruptible(async {
            let message = rx.recv().await.map_err(|_| LinuxError::ECONNRESET)?;
            let mut read = 0;
            loop {
                let chunk = dst.chunk_mut();
                let len = chunk.len().min(message.len() - read);
                if len == 0 {
                    break;
                }

                chunk[..len].copy_from_slice(&message[read..read + len]);
                read += len;
            }
            if read < message.len() {
                warn!("UDP message truncated: {} -> {} bytes", message.len(), read);
            }

            Ok(if flags.contains(RecvFlags::TRUNCATE) {
                message.len()
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
        };
        let transport2 = DgramTransport {
            data_rx: Mutex::new(Some(rx2)),
            connected: RwLock::new(Some(Channel { data_tx: tx1 })),
        };
        Ok((transport1, transport2))
    }
}
