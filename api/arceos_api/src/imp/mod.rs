mod mem;
mod task;

cfg_fs! {
    mod fs;
    pub use fs::*;
}

cfg_net! {
    mod net;
    pub use net::*;
}

cfg_display! {
    mod display;
    pub use display::*;
}

mod stdio {
    use core::fmt;

    pub fn ax_console_read_bytes(buf: &mut [u8]) -> crate::AxResult<usize> {
        let len = axhal::console::read_bytes(buf);
        for c in &mut buf[..len] {
            if *c == b'\r' {
                *c = b'\n';
            }
        }
        Ok(len)
    }

    pub fn ax_console_write_bytes(buf: &[u8]) -> crate::AxResult<usize> {
        axhal::console::write_bytes(buf);
        Ok(buf.len())
    }

    pub fn ax_console_write_fmt(args: fmt::Arguments) -> fmt::Result {
        axlog::print_fmt(args)
    }
}

mod time {
    pub use axhal::time::{
        TimeValue as AxTimeValue, monotonic_time as ax_monotonic_time, wall_time as ax_wall_time,
    };
}

pub use self::mem::*;
pub use self::stdio::*;
pub use self::task::*;
pub use self::time::*;

pub use axhal::misc::terminate as ax_terminate;
pub use axio::PollState as AxPollState;
