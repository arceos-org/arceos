use core::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    task::Waker,
    time::Duration,
};

use axerrno::AxResult;
use axio::{IoEvents, Pollable};
use axtask::future::Poller;

use crate::{
    SERVICE,
    options::{Configurable, GetSocketOption, SetSocketOption},
};

/// General options for all sockets.
pub(crate) struct GeneralOptions {
    /// Whether the socket is non-blocking.
    nonblock: AtomicBool,
    /// Whether the socket should reuse the address.
    reuse_address: AtomicBool,

    send_timeout_nanos: AtomicU64,
    recv_timeout_nanos: AtomicU64,

    device_mask: AtomicU32,
}
impl Default for GeneralOptions {
    fn default() -> Self {
        Self::new()
    }
}
impl GeneralOptions {
    pub fn new() -> Self {
        Self {
            nonblock: AtomicBool::new(false),
            reuse_address: AtomicBool::new(false),

            send_timeout_nanos: AtomicU64::new(0),
            recv_timeout_nanos: AtomicU64::new(0),

            device_mask: AtomicU32::new(0),
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

    pub fn set_device_mask(&self, mask: u32) {
        self.device_mask.store(mask, Ordering::Release);
    }

    pub fn device_mask(&self) -> u32 {
        self.device_mask.load(Ordering::Acquire)
    }

    pub fn register_waker(&self, waker: &Waker) {
        SERVICE
            .lock()
            .register_waker(self.device_mask(), waker);
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
    fn get_option_inner(&self, option: &mut GetSocketOption) -> AxResult<bool> {
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

    fn set_option_inner(&self, option: SetSocketOption) -> AxResult<bool> {
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
