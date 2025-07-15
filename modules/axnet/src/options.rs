use core::time::Duration;

use axerrno::{LinuxError, LinuxResult};
use enum_dispatch::enum_dispatch;

macro_rules! define_options {
    ($($name:ident($value:ty),)*) => {
        /// Operation to get a socket option.
        ///
        /// See [`Configurable::get_option`].
        pub enum GetSocketOption<'a> {
            $(
                $name(&'a mut $value),
            )*
        }

        /// Operation to set a socket option.
        ///
        /// See [`Configurable::set_option`].
        #[derive(Clone, Copy)]
        pub enum SetSocketOption<'a> {
            $(
                $name(&'a $value),
            )*
        }
    };
}

define_options! {
    // ---- Socket level options (SO_*) ----
    ReuseAddress(bool),
    Error(i32),
    DontRoute(bool),
    SendBuffer(usize),
    ReceiveBuffer(usize),
    KeepAlive(bool),
    SendTimeout(Duration),
    ReceiveTimeout(Duration),
    SendBufferForce(usize),

    // --- TCP level options (TCP_*) ----
    NoDelay(bool),
    MaxSegment(usize),
    TcpInfo(()),

    // ---- IP level options (IP_*) ----
    Ttl(u8),

    // ---- Extra options ----
    NonBlocking(bool),
}

/// Trait for configurable socket-like objects.
#[enum_dispatch]
pub trait Configurable {
    /// Get a socket option, returns `true` if the socket supports the option.
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> LinuxResult<bool>;
    /// Set a socket option, returns `true` if the socket supports the option.
    fn set_option_inner(&self, opt: SetSocketOption) -> LinuxResult<bool>;

    fn get_option(&self, mut opt: GetSocketOption) -> LinuxResult<()> {
        self.get_option_inner(&mut opt).and_then(|supported| {
            if !supported {
                Err(LinuxError::ENOPROTOOPT)
            } else {
                Ok(())
            }
        })
    }
    fn set_option(&self, opt: SetSocketOption) -> LinuxResult<()> {
        self.set_option_inner(opt).and_then(|supported| {
            if !supported {
                Err(LinuxError::ENOPROTOOPT)
            } else {
                Ok(())
            }
        })
    }
}
