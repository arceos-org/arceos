use alloc::vec::Vec;
use axerrno::{ax_err_type, AxError, AxResult};

use smoltcp::iface::SocketHandle;
use smoltcp::socket::dns::{self, GetQueryResultError, StartQueryError};
use smoltcp::wire::{DnsQueryType, IpAddress};

use super::{SocketSetWrapper, ETH0, SOCKET_SET};

/// A DNS socket.
struct DnsSocket {
    handle: Option<SocketHandle>,
}

impl DnsSocket {
    #[allow(clippy::new_without_default)]
    /// Creates a new DNS socket.
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_dns_socket();
        let handle = Some(SOCKET_SET.add(socket));
        Self { handle }
    }

    #[allow(dead_code)]
    /// Update the list of DNS servers, will replace all existing servers.
    pub fn update_servers(self, servers: &[smoltcp::wire::IpAddress]) {
        SOCKET_SET.with_socket_mut::<dns::Socket, _, _>(self.handle.unwrap(), |socket| {
            socket.update_servers(servers)
        });
    }

    /// Query a address with given DNS query type.
    pub fn query(&self, name: &str, query_type: DnsQueryType) -> AxResult<Vec<IpAddress>> {
        // let local_addr = self.local_addr.unwrap_or_else(f);
        let handle = self.handle.ok_or_else(|| ax_err_type!(InvalidInput))?;
        let iface = &ETH0.iface;
        let query_handle = SOCKET_SET
            .with_socket_mut::<dns::Socket, _, _>(handle, |socket| {
                socket.start_query(iface.lock().context(), name, query_type)
            })
            .map_err(|e| match e {
                StartQueryError::NoFreeSlot => {
                    ax_err_type!(ResourceBusy, "socket query() failed: no free slot")
                }
                StartQueryError::InvalidName => {
                    ax_err_type!(InvalidInput, "socket query() failed: invalid name")
                }
                StartQueryError::NameTooLong => {
                    ax_err_type!(InvalidInput, "socket query() failed: too long name")
                }
            })?;
        loop {
            SOCKET_SET.poll_interfaces();
            match SOCKET_SET.with_socket_mut::<dns::Socket, _, _>(handle, |socket| {
                socket.get_query_result(query_handle).map_err(|e| match e {
                    GetQueryResultError::Pending => AxError::WouldBlock,
                    GetQueryResultError::Failed => {
                        ax_err_type!(ConnectionRefused, "socket query() failed")
                    }
                })
            }) {
                Ok(n) => {
                    SOCKET_SET.poll_interfaces();
                    return Ok(n.to_vec());
                }
                Err(AxError::WouldBlock) => axtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }
}

impl Drop for DnsSocket {
    fn drop(&mut self) {
        if let Some(handle) = self.handle {
            SOCKET_SET.remove(handle);
        }
    }
}

/// Public function for DNS query.
pub fn resolve_socket_addr(name: &str) -> AxResult<alloc::vec::Vec<IpAddress>> {
    let socket = DnsSocket::new();
    socket.query(name, DnsQueryType::A)
}
