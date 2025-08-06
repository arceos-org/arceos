use alloc::sync::Arc;
use core::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    task::Waker,
    time::Duration,
};

use axerrno::LinuxResult;
use axio::{IoEvents, PollSet, Pollable};
use axtask::future::Poller;

use crate::options::{Configurable, GetSocketOption, SetSocketOption};

/// General options for all sockets.
pub(crate) struct GeneralOptions {
    pub poll_rx: Arc<PollSet>,
    pub poll_tx: Arc<PollSet>,
    pub poll_rx_waker: Waker,
    pub poll_tx_waker: Waker,

    /// Whether the socket is non-blocking.
    nonblock: AtomicBool,
    /// Whether the socket should reuse the address.
    reuse_address: AtomicBool,

    send_timeout_nanos: AtomicU64,
    recv_timeout_nanos: AtomicU64,

    /// Whether the socket is externally driven (e.g., by a network interface).
    ///
    /// This means new data can be received without between two consecutive
    /// calls to `poll_interfaces` (without our intervention between them). This
    /// is not true for loopback devices since they are always driven internally
    /// by kernel.
    externally_driven: AtomicBool,
}
impl Default for GeneralOptions {
    fn default() -> Self {
        Self::new()
    }
}
impl GeneralOptions {
    pub fn new() -> Self {
        let poll_rx = Arc::new(PollSet::new());
        let poll_tx = Arc::new(PollSet::new());
        let poll_rx_waker = Waker::from(poll_rx.clone());
        let poll_tx_waker = Waker::from(poll_tx.clone());
        Self {
            poll_rx,
            poll_tx,
            poll_rx_waker,
            poll_tx_waker,

            nonblock: AtomicBool::new(false),
            reuse_address: AtomicBool::new(false),

            send_timeout_nanos: AtomicU64::new(0),
            recv_timeout_nanos: AtomicU64::new(0),

            externally_driven: AtomicBool::new(false),
        }
    }

    pub fn nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Relaxed)
    }

    pub fn reuse_address(&self) -> bool {
        self.reuse_address.load(Ordering::Relaxed)
    }

    pub fn send_timeout(&self) -> Option<Duration> {
        let nanos = self.send_timeout_nanos.load(Ordering::Relaxed);
        (nanos > 0).then(|| Duration::from_nanos(nanos))
    }

    pub fn recv_timeout(&self) -> Option<Duration> {
        let nanos = self.recv_timeout_nanos.load(Ordering::Relaxed);
        (nanos > 0).then(|| Duration::from_nanos(nanos))
    }

    pub fn set_externally_driven(&self, driven: bool) {
        self.externally_driven.store(driven, Ordering::Release);
    }

    pub fn externally_driven(&self) -> bool {
        self.externally_driven.load(Ordering::Acquire)
    }

    pub fn send_poller<'a, P: Pollable>(&self, pollable: &'a P) -> Poller<'a, P> {
        Poller::new(pollable, IoEvents::OUT)
            .non_blocking(self.nonblocking())
            .timeout(self.send_timeout())
    }

    pub fn recv_poller<'a, P: Pollable>(&self, pollable: &'a P) -> Poller<'a, P> {
        Poller::new(pollable, IoEvents::IN)
            .non_blocking(self.nonblocking())
            .timeout(self.recv_timeout())
    }
}
impl Configurable for GeneralOptions {
    fn get_option_inner(&self, option: &mut GetSocketOption) -> LinuxResult<bool> {
        use GetSocketOption as O;
        match option {
            O::Error(error) => {
                // TODO(mivik): actual logic
                **error = 0;
            }
            O::NonBlocking(nonblock) => {
                **nonblock = self.nonblocking();
            }
            O::ReuseAddress(reuse) => {
                **reuse = self.reuse_address();
            }
            O::SendTimeout(timeout) => {
                **timeout = Duration::from_nanos(self.send_timeout_nanos.load(Ordering::Relaxed));
            }
            O::ReceiveTimeout(timeout) => {
                **timeout = Duration::from_nanos(self.recv_timeout_nanos.load(Ordering::Relaxed));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_option_inner(&self, option: SetSocketOption) -> LinuxResult<bool> {
        use SetSocketOption as O;

        match option {
            O::NonBlocking(nonblock) => {
                self.nonblock.store(*nonblock, Ordering::Relaxed);
            }
            O::ReuseAddress(reuse) => {
                self.reuse_address.store(*reuse, Ordering::Relaxed);
            }
            O::SendTimeout(timeout) => {
                self.send_timeout_nanos
                    .store(timeout.as_nanos() as u64, Ordering::Relaxed);
            }
            O::ReceiveTimeout(timeout) => {
                self.recv_timeout_nanos
                    .store(timeout.as_nanos() as u64, Ordering::Relaxed);
            }
            O::SendBuffer(_) | O::ReceiveBuffer(_) => {
                // TODO(mivik): implement buffer size options
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
