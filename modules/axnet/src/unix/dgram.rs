use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::task::Context;

use async_channel::TryRecvError;
use async_trait::async_trait;
use axerrno::{LinuxError, LinuxResult};
use axio::{Buf, BufMut, IoEvents, PollSet, Pollable};
use axsync::Mutex;
use spin::RwLock;

use crate::{
    CMsgData, RecvFlags, RecvOptions, SendOptions, SocketAddrEx,
    general::GeneralOptions,
    options::{Configurable, GetSocketOption, SetSocketOption, UnixCredentials},
    unix::{Transport, TransportOps, UnixSocketAddr, with_slot},
};

struct Packet {
    data: Vec<u8>,
    cmsg: Vec<CMsgData>,
    sender: UnixSocketAddr,
}

struct Channel {
    data_tx: async_channel::Sender<Packet>,
    poll_update: Arc<PollSet>,
}

pub struct Bind {
    data_tx: async_channel::Sender<Packet>,
    poll_update: Arc<PollSet>,
}
impl Bind {
    fn connect(&self) -> Channel {
        let tx = self.data_tx.clone();
        Channel {
            data_tx: tx,
            poll_update: self.poll_update.clone(),
        }
    }
}

pub struct DgramTransport {
    data_rx: Mutex<Option<(async_channel::Receiver<Packet>, Arc<PollSet>)>>,
    connected: RwLock<Option<Channel>>,
    local_addr: RwLock<UnixSocketAddr>,
    poll_state: Arc<PollSet>,
    general: GeneralOptions,
    pid: u32,
}
impl DgramTransport {
    pub fn new(pid: u32) -> Self {
        DgramTransport {
            data_rx: Mutex::new(None),
            connected: RwLock::new(None),
            local_addr: RwLock::new(UnixSocketAddr::Unnamed),
            poll_state: Arc::default(),
            general: GeneralOptions::default(),
            pid,
        }
    }

    fn new_connected(
        data_rx: (async_channel::Receiver<Packet>, Arc<PollSet>),
        connected: Channel,
        pid: u32,
    ) -> Self {
        DgramTransport {
            data_rx: Mutex::new(Some(data_rx)),
            connected: RwLock::new(Some(connected)),
            local_addr: RwLock::new(UnixSocketAddr::Unnamed),
            poll_state: Arc::default(),
            general: GeneralOptions::default(),
            pid,
        }
    }

    pub fn new_pair(pid: u32) -> (Self, Self) {
        let (tx1, rx1) = async_channel::unbounded();
        let (tx2, rx2) = async_channel::unbounded();
        let poll1 = Arc::new(PollSet::new());
        let poll2 = Arc::new(PollSet::new());
        let transport1 = DgramTransport::new_connected(
            (rx1, poll1.clone()),
            Channel {
                data_tx: tx2,
                poll_update: poll2.clone(),
            },
            pid,
        );
        let transport2 = DgramTransport::new_connected(
            (rx2, poll2.clone()),
            Channel {
                data_tx: tx1,
                poll_update: poll1.clone(),
            },
            pid,
        );
        (transport1, transport2)
    }
}

impl Configurable for DgramTransport {
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> LinuxResult<bool> {
        use GetSocketOption as O;

        if self.general.get_option_inner(opt)? {
            return Ok(true);
        }

        match opt {
            O::PassCredentials(_) => {}
            O::PeerCredentials(cred) => {
                // Datagram sockets are stateless and do not have a peer, so we
                // return the credentials of the process that created the
                // socket.
                **cred = UnixCredentials::new(self.pid);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_option_inner(&self, opt: SetSocketOption) -> LinuxResult<bool> {
        use SetSocketOption as O;

        if self.general.set_option_inner(opt)? {
            return Ok(true);
        }

        match opt {
            O::PassCredentials(_) => {}
            _ => return Ok(false),
        }
        Ok(true)
    }
}
#[async_trait]
impl TransportOps for DgramTransport {
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
        let poll_update = Arc::new(PollSet::new());
        *slot = Some(Bind {
            data_tx: tx,
            poll_update: poll_update.clone(),
        });
        *guard = Some((rx, poll_update));
        self.local_addr.write().clone_from(local_addr);
        self.poll_state.wake();
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
        self.poll_state.wake();
        Ok(())
    }

    async fn accept(&self) -> LinuxResult<(Transport, UnixSocketAddr)> {
        Err(LinuxError::EINVAL)
    }

    fn send(&self, src: &mut impl Buf, options: SendOptions) -> LinuxResult<usize> {
        let mut message = Vec::new();
        src.read_to_end(&mut message)?;
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
                    bind.poll_update.wake();
                    Ok(())
                } else {
                    Err(LinuxError::ENOTCONN)
                }
            })?;
        } else if let Some(chan) = connected.as_ref() {
            chan.data_tx
                .try_send(packet)
                .map_err(|_| LinuxError::EPIPE)?;
            chan.poll_update.wake();
        } else {
            return Err(LinuxError::ENOTCONN);
        }
        Ok(len)
    }

    fn recv(&self, dst: &mut impl BufMut, mut options: RecvOptions) -> LinuxResult<usize> {
        self.general.recv_poller(self).poll(move || {
            let mut guard = self.data_rx.lock();
            let Some((rx, _)) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };

            let Packet { data, cmsg, sender } = match rx.try_recv() {
                Ok(packet) => packet,
                Err(TryRecvError::Empty) => {
                    return Err(LinuxError::EAGAIN);
                }
                Err(TryRecvError::Closed) => {
                    return Ok(0);
                }
            };
            let count = dst.write(&data)?;
            if count < data.len() {
                warn!("UDP message truncated: {} -> {} bytes", data.len(), count);
            }

            if let Some(from) = options.from.as_mut() {
                **from = SocketAddrEx::Unix(sender);
            }
            if let Some(dst) = options.cmsg.as_mut() {
                dst.extend(cmsg);
            }

            Ok(if options.flags.contains(RecvFlags::TRUNCATE) {
                data.len()
            } else {
                count
            })
        })
    }
}

impl Pollable for DgramTransport {
    fn poll(&self) -> IoEvents {
        let mut events = IoEvents::OUT;
        if let Some((rx, _)) = self.data_rx.lock().as_ref() {
            events.set(IoEvents::IN, !rx.is_empty());
        }
        events
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        if let Some((_, poll)) = self.data_rx.lock().as_ref() {
            if events.contains(IoEvents::IN) {
                poll.register(context.waker());
            }
        }
    }
}

impl Drop for DgramTransport {
    fn drop(&mut self) {
        if let Some(chan) = self.connected.write().take() {
            chan.poll_update.wake();
        }
    }
}
