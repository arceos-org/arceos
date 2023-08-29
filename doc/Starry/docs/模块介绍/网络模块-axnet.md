
axnet 提供了对网络设备的驱动与封装。StarryOS 使用了开源的 smoltcp 库作为其网络协议栈。并在 smoltcp 提供的 API 基础上，封装了 TCPSocket 与 UDPSocket 两个套接字类型。

Linux syscall 所提供的 Socket API 的实现位于 ulib/socket.rs 中。

```rust
/// 包装内部的不同协议 Socket
pub struct Socket {
    #[allow(dead_code)]
    domain: Domain,
    socket_type: SocketType,
    inner: SocketInner,
    close_exec: bool,
    recv_timeout: Option<TimeVal>,

    reuse_addr: bool,
    dont_route: bool,
    send_buf_size: usize,
    recv_buf_size: usize,
    congestion: String,
}
```

### Socket

Socket 封装了由 axnet 提供的 TCP/UDP Socket（SocketInner）。并记录了此 Socket 在 Linux Socket API 层具有的其他信息，如 domain（Address Family）、type，以及一些配置信息，如 recv_timeout、close_exec。

此 Socket 类型实现了如 bind()、connect()、accept() 等操作，再 syscall 的实际代码中，只需使用这些函数，无需与 axnet 交互。

Socket 与其他文件相关的类型相同，实现了 FileExt 与 FileIO trait，使得 Socket 对象可以存储在进程的 FdTable 中。也因如此，read()、write() 等面向文件实现的 syscall 也可用于 socket。

### SocketOption、TcpSocketOption

Linux Socket API 中关于，可以通过 Socket Option 来修改或获取 Socket 的各项选项。对于不同的选项，分别实现了 get() 与 set() 函数，简洁地实现所需功能。

```rust
#[derive(TryFromPrimitive, Debug)]
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum SocketOption {
    SO_REUSEADDR = 2,
    SO_DONTROUTE = 5,
    SO_SNDBUF = 7,
    SO_RCVBUF = 8,
    SO_KEEPALIVE = 9,
    SO_RCVTIMEO = 20,
}
```
