//! 提供与 net work 相关的 syscall
#![cfg_attr(all(not(test), not(doc)), no_std)]
use syscall_utils::SyscallResult;
mod imp;

#[allow(unused)]
mod socket;
use imp::*;
mod net_syscall_id;
pub use net_syscall_id::NetSyscallId::{self, *};
pub use socket::Socket;
/// 进行 syscall 的分发
pub fn net_syscall(syscall_id: net_syscall_id::NetSyscallId, args: [usize; 6]) -> SyscallResult {
    match syscall_id {
        SOCKET => syscall_socket(args[0], args[1], args[2]),
        BIND => syscall_bind(args[0], args[1] as *const u8, args[2]),
        LISTEN => syscall_listen(args[0], args[1]),
        ACCEPT => syscall_accept4(args[0], args[1] as *mut u8, args[2] as *mut u32, 0),
        CONNECT => syscall_connect(args[0], args[1] as *const u8, args[2]),
        GETSOCKNAME => syscall_get_sock_name(args[0], args[1] as *mut u8, args[2] as *mut u32),
        GETPEERNAME => syscall_getpeername(args[0], args[1] as *mut u8, args[2] as *mut u32),
        // GETPEERNAME => 0,
        SENDTO => syscall_sendto(
            args[0],
            args[1] as *const u8,
            args[2],
            args[3],
            args[4] as *const u8,
            args[5],
        ),
        RECVFROM => syscall_recvfrom(
            args[0],
            args[1] as *mut u8,
            args[2],
            args[3],
            args[4] as *mut u8,
            args[5] as *mut u32,
        ),
        SETSOCKOPT => syscall_set_sock_opt(
            args[0],
            args[1],
            args[2],
            args[3] as *const u8,
            args[4] as u32,
        ),
        // SETSOCKOPT => 0,
        GETSOCKOPT => syscall_get_sock_opt(
            args[0],
            args[1],
            args[2],
            args[3] as *mut u8,
            args[4] as *mut u32,
        ),
        ACCEPT4 => syscall_accept4(args[0], args[1] as *mut u8, args[2] as *mut u32, args[3]),
        SHUTDOWN => syscall_shutdown(args[0], args[1]),
        #[allow(unused)]
        _ => {
            panic!("Invalid Syscall Id: {:?}!", syscall_id);
            // return -1;
            // exit(-1)
        }
    }
}
