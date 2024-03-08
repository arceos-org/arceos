//! 记录该模块使用到的系统调用 id
//!
//!
#[cfg(any(
    target_arch = "riscv32",
    target_arch = "riscv64",
    target_arch = "aarch64"
))]
numeric_enum_macro::numeric_enum! {
#[repr(usize)]
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum NetSyscallId {
    // Socket
    SOCKET = 198,
    SOCKETPAIR = 199,
    BIND = 200,
    LISTEN = 201,
    ACCEPT = 202,
    CONNECT = 203,
    GETSOCKNAME = 204,
    GETPEERNAME = 205,
    SENDTO = 206,
    RECVFROM = 207,
    SETSOCKOPT = 208,
    GETSOCKOPT = 209,
    SHUTDOWN = 210,
    ACCEPT4 = 242,
}
}

#[cfg(target_arch = "x86_64")]
numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[allow(missing_docs)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum NetSyscallId {
        // Socket
        SOCKET = 41,
        BIND = 49,
        LISTEN = 50,
        ACCEPT = 43,
        CONNECT = 42,
        GETSOCKNAME = 51,
        GETPEERNAME = 52,
        SOCKETPAIR = 53,
        SENDTO = 44,
        RECVFROM = 45,
        SETSOCKOPT = 54,
        GETSOCKOPT = 55,
        SHUTDOWN = 48,
        ACCEPT4 = 288,
    }
}
