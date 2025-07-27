use core::{
    future::poll_fn,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use axerrno::{LinuxError, LinuxResult};
use axtask::future::try_block_on;

use crate::{
    options::{Configurable, GetSocketOption, SetSocketOption},
    poll_interfaces,
};

/// General options for all sockets.
pub(crate) struct GeneralOptions {
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
impl GeneralOptions {
    pub const fn new() -> Self {
        Self {
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

    pub fn block_on<F, T>(&self, timeout: Option<Duration>, mut f: F) -> LinuxResult<T>
    where
        F: FnMut(&mut Context) -> Poll<LinuxResult<T>> + Unpin,
    {
        if self.nonblocking() {
            let mut context = Context::from_waker(Waker::noop());
            match f(&mut context) {
                Poll::Ready(result) => result,
                Poll::Pending => Err(LinuxError::EAGAIN),
            }
        } else {
            // Linux manual:
            // The following interfaces are never restarted after being
            //    interrupted by a signal handler, regardless of the use of
            //    SA_RESTART; they always fail with the error EINTR when interrupted
            //    by a signal handler:
            // ... socket interfaces, when a timeout has been set.
            let fut = {
                let externally_driven = self.externally_driven.load(Ordering::Acquire);
                poll_fn(move |context| {
                    poll_interfaces();
                    match f(context) {
                        Poll::Ready(Err(LinuxError::EAGAIN)) => {
                            context.waker().wake_by_ref();
                            Poll::Pending
                        }
                        Poll::Ready(result) => return Poll::Ready(result),
                        Poll::Pending => {
                            if externally_driven {
                                context.waker().wake_by_ref();
                            }
                            return Poll::Pending;
                        }
                    }
                })
            };
            let result = if let Some(timeout) = timeout {
                try_block_on(async move {
                    axtask::future::timeout(fut, timeout)
                        .await
                        .ok_or(LinuxError::ETIMEDOUT)?
                })
            } else {
                try_block_on(fut)
            };
            result.transpose().unwrap_or(Err(LinuxError::EINTR))
        }
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
