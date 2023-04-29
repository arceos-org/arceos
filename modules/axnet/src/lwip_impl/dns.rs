use crate::IpAddr;
use axerrno::{ax_err, AxResult};

pub fn resolve_socket_addr(_name: &str) -> AxResult<alloc::vec::Vec<IpAddr>> {
    ax_err!(Unsupported, "LWIP Unsupported")
}
