use alloc::boxed::Box;

use async_trait::async_trait;
use axerrno::{LinuxError, LinuxResult};
use axio::{
    PollState,
    buf::{Buf, BufMut},
};
use axsync::Mutex;
use axtask::future::block_on_interruptible;
use event_listener::Event;
use ringbuf::{
    Arc, HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};

use crate::{
    RecvFlags, SendFlags,
    options::{Configurable, GetSocketOption, SetSocketOption},
    unix::{Transport, UnixSocketAddr},
};

fn new_uni_channel() -> (HeapProd<u8>, HeapCons<u8>) {
    let rb = HeapRb::new(1024 * 1024);
    rb.split()
}
fn new_channels() -> (Channel, Channel) {
    let (client_tx, server_rx) = new_uni_channel();
    let (server_tx, client_rx) = new_uni_channel();
    let event = Arc::new(Event::new());
    (
        Channel {
            tx: client_tx,
            rx: client_rx,
            event: event.clone(),
        },
        Channel {
            tx: server_tx,
            rx: server_rx,
            event,
        },
    )
}

struct Channel {
    tx: HeapProd<u8>,
    rx: HeapCons<u8>,
    // TODO: granularity
    event: Arc<Event>,
}

pub struct Bind {
    /// New connections are sent to this channel.
    conn_tx: async_channel::Sender<(Channel, UnixSocketAddr)>,
}
impl Bind {
    fn connect(&self, local_addr: UnixSocketAddr) -> LinuxResult<Channel> {
        let (client_chan, server_chan) = new_channels();
        self.conn_tx
            .try_send((server_chan, local_addr))
            .map_err(|_| LinuxError::ECONNREFUSED)?;
        Ok(client_chan)
    }
}

pub struct StreamTransport {
    channel: Mutex<Option<Channel>>,
    conn_rx: Mutex<Option<async_channel::Receiver<(Channel, UnixSocketAddr)>>>,
}
impl StreamTransport {
    pub fn new() -> Self {
        StreamTransport {
            channel: Mutex::new(None),
            conn_rx: Mutex::new(None),
        }
    }
}

impl Configurable for StreamTransport {
    fn get_option_inner(&self, _opt: &mut GetSocketOption) -> LinuxResult<bool> {
        Ok(false)
    }

    fn set_option_inner(&self, _opt: SetSocketOption) -> LinuxResult<bool> {
        Ok(false)
    }
}
#[async_trait]
impl Transport for StreamTransport {
    fn bind(&self, slot: &super::BindSlot) -> LinuxResult<()> {
        let mut slot = slot.stream.lock();
        if slot.is_some() {
            return Err(LinuxError::EADDRINUSE);
        }
        let mut guard = self.conn_rx.lock();
        if guard.is_some() {
            return Err(LinuxError::EINVAL);
        }
        let (tx, rx) = async_channel::unbounded();
        *slot = Some(Bind { conn_tx: tx });
        *guard = Some(rx);
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
        Ok(())
    }

    async fn accept(&self) -> LinuxResult<(Box<dyn super::Transport>, UnixSocketAddr)> {
        let mut guard = self.conn_rx.lock();
        let Some(rx) = guard.as_mut() else {
            return Err(LinuxError::ENOTCONN);
        };
        let (channel, peer_addr) = rx.recv().await.map_err(|_| LinuxError::ECONNRESET)?;
        Ok((
            Box::new(StreamTransport {
                channel: Mutex::new(Some(channel)),
                conn_rx: Mutex::new(None),
            }) as Box<dyn super::Transport>,
            peer_addr,
        ))
    }

    fn send(
        &self,
        src: &mut dyn Buf,
        to: Option<UnixSocketAddr>,
        _flags: SendFlags,
    ) -> LinuxResult<usize> {
        if to.is_some() {
            return Err(LinuxError::EINVAL);
        }
        let mut total = 0;
        loop {
            let mut guard = self.channel.lock();
            let Some(chan) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };

            let chunk = src.chunk();
            if chunk.is_empty() {
                break;
            }

            let written = chan.tx.push_slice(chunk);
            if written > 0 {
                total += written;
                src.advance(written);
                chan.event.notify(usize::MAX);
                continue;
            }

            let listener = chan.event.listen();

            let written = chan.tx.push_slice(chunk);
            if written > 0 {
                total += written;
                src.advance(written);
                chan.event.notify(usize::MAX);
                continue;
            }

            drop(guard);
            block_on_interruptible(async {
                listener.await;
                Ok(())
            })?;
        }
        Ok(total)
    }

    fn recv(&self, dst: &mut dyn BufMut, _flags: RecvFlags) -> LinuxResult<usize> {
        loop {
            let mut guard = self.channel.lock();
            let Some(chan) = guard.as_mut() else {
                return Err(LinuxError::ENOTCONN);
            };

            let chunk = dst.chunk_mut();
            if chunk.is_empty() {
                return Ok(0);
            }

            let read = chan.rx.pop_slice(chunk);
            if read > 0 {
                dst.advance(read);
                return Ok(read);
            }

            let listener = chan.event.listen();

            let read = chan.rx.pop_slice(chunk);
            if read > 0 {
                dst.advance(read);
                return Ok(read);
            }

            drop(guard);
            block_on_interruptible(async {
                listener.await;
                Ok(())
            })?;
        }
    }

    fn poll(&self) -> LinuxResult<PollState> {
        if let Some(chan) = self.channel.lock().as_ref() {
            Ok(PollState {
                readable: chan.rx.occupied_len() > 0,
                writable: chan.tx.vacant_len() > 0,
            })
        } else if let Some(conn_tx) = self.conn_rx.lock().as_ref() {
            Ok(PollState {
                readable: conn_tx.len() > 0,
                writable: false,
            })
        } else {
            Err(LinuxError::ENOTCONN)
        }
    }

    fn make_pair() -> LinuxResult<(Self, Self)> {
        let (chan1, chan2) = new_channels();
        let transport1 = StreamTransport {
            channel: Mutex::new(Some(chan1)),
            conn_rx: Mutex::new(None),
        };
        let transport2 = StreamTransport {
            channel: Mutex::new(Some(chan2)),
            conn_rx: Mutex::new(None),
        };
        Ok((transport1, transport2))
    }
}
