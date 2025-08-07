use alloc::{boxed::Box, sync::Arc};
use core::{
    convert::Infallible,
    sync::atomic::{AtomicBool, Ordering},
    task::Context,
};

use async_trait::async_trait;
use axerrno::{LinuxError, LinuxResult};
use axio::{
    IoEvents, PollSet, Pollable,
    buf::{Buf, BufExt, BufMut, BufMutExt},
};
use axsync::Mutex;
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};

use crate::{
    RecvOptions, SendOptions, Shutdown,
    general::GeneralOptions,
    options::{Configurable, GetSocketOption, SetSocketOption, UnixCredentials},
    unix::{Transport, TransportOps, UnixSocketAddr},
};

const BUF_SIZE: usize = 64 * 1024;

fn new_uni_channel() -> (HeapProd<u8>, HeapCons<u8>) {
    let rb = HeapRb::new(BUF_SIZE);
    rb.split()
}
fn new_channels(pid: u32) -> (Channel, Channel) {
    let (client_tx, server_rx) = new_uni_channel();
    let (server_tx, client_rx) = new_uni_channel();
    let poll_update = Arc::new(PollSet::new());
    (
        Channel {
            tx: client_tx,
            rx: client_rx,
            poll_update: poll_update.clone(),
            peer_pid: pid,
        },
        Channel {
            tx: server_tx,
            rx: server_rx,
            poll_update,
            peer_pid: pid,
        },
    )
}

struct Channel {
    tx: HeapProd<u8>,
    rx: HeapCons<u8>,
    // TODO: granularity
    poll_update: Arc<PollSet>,
    peer_pid: u32,
}

pub struct Bind {
    /// New connections are sent to this channel.
    conn_tx: async_channel::Sender<ConnRequest>,
    poll_new_conn: Arc<PollSet>,
    pid: u32,
}
impl Bind {
    fn connect(&self, local_addr: UnixSocketAddr, pid: u32) -> LinuxResult<Channel> {
        let (mut client_chan, mut server_chan) = new_channels(0);
        client_chan.peer_pid = self.pid;
        server_chan.peer_pid = pid;
        self.conn_tx
            .try_send(ConnRequest {
                channel: server_chan,
                addr: local_addr,
                pid,
            })
            .map_err(|_| LinuxError::ECONNREFUSED)?;
        self.poll_new_conn.wake();
        Ok(client_chan)
    }
}

struct ConnRequest {
    channel: Channel,
    addr: UnixSocketAddr,
    pid: u32,
}

pub struct StreamTransport {
    channel: Mutex<Option<Channel>>,
    conn_rx: Mutex<Option<(async_channel::Receiver<ConnRequest>, Arc<PollSet>)>>,
    poll_state: PollSet,
    general: GeneralOptions,
    pid: u32,
    rx_closed: AtomicBool,
    tx_closed: AtomicBool,
}
impl StreamTransport {
    pub fn new(pid: u32) -> Self {
        StreamTransport::new_channel(None, pid)
    }

    fn new_channel(channel: Option<Channel>, pid: u32) -> Self {
        StreamTransport {
            channel: Mutex::new(channel),
            conn_rx: Mutex::new(None),
            poll_state: PollSet::new(),
            general: GeneralOptions::default(),
            pid,
            rx_closed: AtomicBool::new(false),
            tx_closed: AtomicBool::new(false),
        }
    }

    pub fn new_pair(pid: u32) -> (Self, Self) {
        let (chan1, chan2) = new_channels(pid);
        let transport1 = StreamTransport::new_channel(Some(chan1), pid);
        let transport2 = StreamTransport::new_channel(Some(chan2), pid);
        (transport1, transport2)
    }
}

impl Configurable for StreamTransport {
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> LinuxResult<bool> {
        use GetSocketOption as O;

        if self.general.get_option_inner(opt)? {
            return Ok(true);
        }

        match opt {
            O::SendBuffer(size) => {
                **size = BUF_SIZE;
            }
            O::PassCredentials(_) => {}
            O::PeerCredentials(cred) => {
                let peer_pid = self
                    .channel
                    .lock()
                    .as_ref()
                    .map_or(self.pid, |chan| chan.peer_pid);
                **cred = UnixCredentials::new(peer_pid);
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
impl TransportOps for StreamTransport {
    fn bind(&self, slot: &super::BindSlot, _local_addr: &UnixSocketAddr) -> LinuxResult<()> {
        let mut slot = slot.stream.lock();
        if slot.is_some() {
            return Err(LinuxError::EADDRINUSE);
        }
        let mut guard = self.conn_rx.lock();
        if guard.is_some() {
            return Err(LinuxError::EINVAL);
        }
        let (tx, rx) = async_channel::unbounded();
        let poll = Arc::new(PollSet::new());
        *slot = Some(Bind {
            conn_tx: tx,
            poll_new_conn: poll.clone(),
            pid: self.pid,
        });
        *guard = Some((rx, poll));
        self.poll_state.wake();
        Ok(())
    }

    fn connect(&self, slot: &super::BindSlot, local_addr: &UnixSocketAddr) -> LinuxResult<()> {
        let mut guard = self.channel.lock();
        if guard.is_some() {
            return Err(LinuxError::EISCONN);
        }
        *guard = Some(
            slot.stream
                .lock()
                .as_ref()
                .ok_or(LinuxError::ENOTCONN)?
                .connect(local_addr.clone(), self.pid)?,
        );
        self.poll_state.wake();
        Ok(())
    }

    async fn accept(&self) -> LinuxResult<(Transport, UnixSocketAddr)> {
        let mut guard = self.conn_rx.lock();
        let Some((rx, _)) = guard.as_mut() else {
            return Err(LinuxError::ENOTCONN);
        };
        let ConnRequest {
            channel,
            addr: peer_addr,
            pid,
        } = rx.recv().await.map_err(|_| LinuxError::ECONNRESET)?;
        Ok((
            Transport::Stream(StreamTransport::new_channel(Some(channel), pid)),
            peer_addr,
        ))
    }

    fn send(&self, src: &mut impl Buf, options: SendOptions) -> LinuxResult<usize> {
        if options.to.is_some() {
            return Err(LinuxError::EINVAL);
        }
        let mut total = 0;
        self.general.send_poller(self).poll(|| {
            let mut guard = self.channel.lock();
            let Some(chan) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };
            if !chan.tx.read_is_held() {
                return Err(LinuxError::EPIPE);
            }

            let written = src
                .read_with::<Infallible>(|chunk| Ok(chan.tx.push_slice(chunk)))
                .map_err(|err| match err {})?;
            total += written;
            if written > 0 {
                chan.poll_update.wake();
            }

            if src.chunk().is_empty() {
                Ok(total)
            } else {
                Err(LinuxError::EAGAIN)
            }
        })
    }

    fn recv(&self, dst: &mut impl BufMut, _options: RecvOptions) -> LinuxResult<usize> {
        self.general.recv_poller(self).poll(|| {
            let mut guard = self.channel.lock();
            let Some(chan) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };

            let read = dst
                .fill_with::<Infallible>(|chunk| Ok(chan.rx.pop_slice(chunk)))
                .map_err(|err| match err {})?;
            if read > 0 {
                chan.poll_update.wake();
                Ok(read)
            } else {
                Err(LinuxError::EAGAIN)
            }
        })
    }

    fn shutdown(&self, how: Shutdown) -> LinuxResult<()> {
        if how.has_read() {
            self.rx_closed.store(true, Ordering::Release);
            self.poll_state.wake();
        }
        if how.has_write() {
            self.tx_closed.store(true, Ordering::Release);
            self.poll_state.wake();
        }
        if self.rx_closed.load(Ordering::Acquire) && self.tx_closed.load(Ordering::Acquire) {
            if let Some(chan) = self.channel.lock().take() {
                chan.poll_update.wake();
            }
        }
        Ok(())
    }
}

impl Pollable for StreamTransport {
    fn poll(&self) -> IoEvents {
        let mut events = IoEvents::empty();
        if let Some(chan) = self.channel.lock().as_ref() {
            events.set(
                IoEvents::IN,
                !self.rx_closed.load(Ordering::Acquire) && chan.rx.occupied_len() > 0,
            );
            events.set(
                IoEvents::OUT,
                !self.tx_closed.load(Ordering::Acquire) && chan.tx.vacant_len() > 0,
            );
        } else if let Some((conn_tx, _)) = self.conn_rx.lock().as_ref() {
            events.set(IoEvents::IN, conn_tx.len() > 0);
        }
        events.set(IoEvents::RDHUP, self.rx_closed.load(Ordering::Acquire));
        events
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        if let Some(chan) = self.channel.lock().as_ref() {
            if events.intersects(IoEvents::IN | IoEvents::OUT) {
                chan.poll_update.register(context.waker());
            }
        } else if let Some((_, poll_new_conn)) = self.conn_rx.lock().as_ref() {
            if events.contains(IoEvents::IN) {
                poll_new_conn.register(context.waker());
            }
        }
        self.poll_state.register(context.waker());
    }
}

impl Drop for StreamTransport {
    fn drop(&mut self) {
        if let Some(chan) = self.channel.lock().as_ref() {
            chan.poll_update.wake();
        }
    }
}
