use alloc::boxed::Box;
use core::{convert::Infallible, task::Context};

use async_trait::async_trait;
use axerrno::{LinuxError, LinuxResult};
use axio::{
    IoEvents, PollSet, Pollable,
    buf::{Buf, BufExt, BufMut, BufMutExt},
};
use axsync::Mutex;
use axtask::future::Poller;
use ringbuf::{
    Arc, HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};

use crate::{
    RecvOptions, SendOptions,
    options::{Configurable, GetSocketOption, SetSocketOption},
    unix::{Transport, TransportOps, UnixSocketAddr},
};

const BUF_SIZE: usize = 64 * 1024;

fn new_uni_channel() -> (HeapProd<u8>, HeapCons<u8>) {
    let rb = HeapRb::new(BUF_SIZE);
    rb.split()
}
fn new_channels() -> (Channel, Channel) {
    let (client_tx, server_rx) = new_uni_channel();
    let (server_tx, client_rx) = new_uni_channel();
    let poll_update = Arc::new(PollSet::new());
    (
        Channel {
            tx: client_tx,
            rx: client_rx,
            poll_update: poll_update.clone(),
        },
        Channel {
            tx: server_tx,
            rx: server_rx,
            poll_update,
        },
    )
}

struct Channel {
    tx: HeapProd<u8>,
    rx: HeapCons<u8>,
    // TODO: granularity
    poll_update: Arc<PollSet>,
}

pub struct Bind {
    /// New connections are sent to this channel.
    conn_tx: async_channel::Sender<(Channel, UnixSocketAddr)>,
    poll_new_conn: Arc<PollSet>,
}
impl Bind {
    fn connect(&self, local_addr: UnixSocketAddr) -> LinuxResult<Channel> {
        let (client_chan, server_chan) = new_channels();
        self.conn_tx
            .try_send((server_chan, local_addr))
            .map_err(|_| LinuxError::ECONNREFUSED)?;
        self.poll_new_conn.wake();
        Ok(client_chan)
    }
}

pub struct StreamTransport {
    channel: Mutex<Option<Channel>>,
    conn_rx: Mutex<
        Option<(
            async_channel::Receiver<(Channel, UnixSocketAddr)>,
            Arc<PollSet>,
        )>,
    >,
    poll_state: PollSet,
}
impl StreamTransport {
    pub fn new() -> Self {
        StreamTransport {
            channel: Mutex::new(None),
            conn_rx: Mutex::new(None),
            poll_state: PollSet::new(),
        }
    }

    pub fn new_pair() -> (Self, Self) {
        let (chan1, chan2) = new_channels();
        let transport1 = StreamTransport {
            channel: Mutex::new(Some(chan1)),
            conn_rx: Mutex::new(None),
            poll_state: PollSet::new(),
        };
        let transport2 = StreamTransport {
            channel: Mutex::new(Some(chan2)),
            conn_rx: Mutex::new(None),
            poll_state: PollSet::new(),
        };
        (transport1, transport2)
    }
}

impl Configurable for StreamTransport {
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> LinuxResult<bool> {
        use GetSocketOption as O;

        match opt {
            O::SendBuffer(size) => {
                **size = BUF_SIZE;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_option_inner(&self, _opt: SetSocketOption) -> LinuxResult<bool> {
        Ok(false)
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
                .connect(local_addr.clone())?,
        );
        self.poll_state.wake();
        Ok(())
    }

    async fn accept(&self) -> LinuxResult<(Transport, UnixSocketAddr)> {
        let mut guard = self.conn_rx.lock();
        let Some((rx, _)) = guard.as_mut() else {
            return Err(LinuxError::ENOTCONN);
        };
        let (channel, peer_addr) = rx.recv().await.map_err(|_| LinuxError::ECONNRESET)?;
        Ok((
            Transport::Stream(StreamTransport {
                channel: Mutex::new(Some(channel)),
                conn_rx: Mutex::new(None),
                poll_state: PollSet::new(),
            }),
            peer_addr,
        ))
    }

    fn send(&self, src: &mut impl Buf, options: SendOptions) -> LinuxResult<usize> {
        if options.to.is_some() {
            return Err(LinuxError::EINVAL);
        }
        let mut total = 0;
        Poller::new(self, IoEvents::OUT).poll(|| {
            let mut guard = self.channel.lock();
            let Some(chan) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };
            if !chan.tx.read_is_held() {
                return Err(LinuxError::EPIPE);
            }

            total += src
                .read_with::<Infallible>(|chunk| Ok(chan.tx.push_slice(chunk)))
                .map_err(|err| match err {})?;

            if src.chunk().is_empty() {
                Ok(total)
            } else {
                Err(LinuxError::EAGAIN)
            }
        })
    }

    fn recv(&self, dst: &mut impl BufMut, _options: RecvOptions) -> LinuxResult<usize> {
        Poller::new(self, IoEvents::IN).poll(|| {
            let mut guard = self.channel.lock();
            let Some(chan) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };

            let read = dst
                .fill_with::<Infallible>(|chunk| Ok(chan.rx.pop_slice(chunk)))
                .map_err(|err| match err {})?;

            if dst.chunk_mut().is_empty() || !chan.rx.write_is_held() {
                Ok(read)
            } else {
                Err(LinuxError::EAGAIN)
            }
        })
    }
}

impl Pollable for StreamTransport {
    fn poll(&self) -> IoEvents {
        let mut events = IoEvents::empty();
        if let Some(chan) = self.channel.lock().as_ref() {
            events.set(IoEvents::IN, chan.rx.occupied_len() > 0);
            events.set(IoEvents::OUT, chan.tx.vacant_len() > 0);
        } else if let Some((conn_tx, _)) = self.conn_rx.lock().as_ref() {
            events.set(IoEvents::IN, conn_tx.len() > 0);
        }
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
