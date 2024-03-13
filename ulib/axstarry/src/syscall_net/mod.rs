//! 提供与 net work 相关的 syscall

use crate::SyscallResult;
mod imp;

#[allow(unused)]
mod socket;
use imp::*;
pub use socket::Socket;
mod net_syscall_id;
pub use net_syscall_id::NetSyscallId::{self, *};

/// 进行 syscall 的分发
pub fn net_syscall(syscall_id: net_syscall_id::NetSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        SOCKET => syscall_socket(args),
        BIND => syscall_bind(args),
        LISTEN => syscall_listen(args),
        ACCEPT => syscall_accept4(args),
        CONNECT => syscall_connect(args),
        GETSOCKNAME => syscall_get_sock_name(args),
        GETPEERNAME => syscall_getpeername(args),
        // GETPEERNAME => 0,
        SENDTO => syscall_sendto(args),
        RECVFROM => syscall_recvfrom(args),
        SETSOCKOPT => syscall_set_sock_opt(args),
        // SETSOCKOPT => 0,
        GETSOCKOPT => syscall_get_sock_opt(args),
        SOCKETPAIR => syscall_socketpair(),
        ACCEPT4 => syscall_accept4(args),
        SHUTDOWN => syscall_shutdown(args),
        #[allow(unused)]
        _ => {
            panic!("Invalid Syscall Id: {:?}!", syscall_id);
            // return -1;
            // exit(-1)
        }
    }
}
