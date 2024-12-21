//! Public APIs and types for [ArceOS] modules
//!
//! [ArceOS]: https://github.com/arceos-org/arceos

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(doc_cfg)]
#![allow(unused_imports)]

#[cfg(any(
    feature = "alloc",
    feature = "fs",
    feature = "net",
    feature = "multitask",
    feature = "dummy-if-not-enabled"
))]
extern crate alloc;

#[macro_use]
mod macros;
mod imp;

pub use axerrno::{AxError, AxResult};

/// Platform-specific constants and parameters.
pub mod config {
    pub use axconfig::*;
}

/// System operations.
pub mod sys {
    define_api! {
        /// Shutdown the whole system and all CPUs.
        pub fn ax_terminate() -> !;
    }
}

/// Time-related operations.
pub mod time {
    define_api_type! {
        pub type AxTimeValue;
    }

    define_api! {
        /// Returns the time elapsed since system boot.
        pub fn ax_monotonic_time() -> AxTimeValue;
        /// Returns the time elapsed since epoch, also known as realtime.
        pub fn ax_wall_time() -> AxTimeValue;
    }
}

/// Memory management.
pub mod mem {
    use core::{alloc::Layout, ptr::NonNull};

    define_api! {
        @cfg "alloc";
        /// Allocates a continuous memory blocks with the given `layout` in
        /// the global allocator.
        ///
        /// Returns [`None`] if the allocation fails.
        ///
        /// # Safety
        ///
        /// This function is unsafe because it requires users to manually manage
        /// the buffer life cycle.
        pub unsafe fn ax_alloc(layout: Layout) -> Option<NonNull<u8>>;
        /// Deallocates the memory block at the given `ptr` pointer with the given
        /// `layout`, which should be allocated by [`ax_alloc`].
        ///
        /// # Safety
        ///
        /// This function is unsafe because it requires users to manually manage
        /// the buffer life cycle.
        pub unsafe fn ax_dealloc(ptr: NonNull<u8>, layout: Layout);
    }

    define_api_type! {
        @cfg "dma";
        pub type DMAInfo;
    }

    define_api! {
        @cfg "dma";
        /// Allocates **coherent** memory that meets Direct Memory Access (DMA)
        /// requirements.
        ///
        /// Returns [`None`] if the allocation fails.
        ///
        /// # Safety
        ///
        /// This function is unsafe because it requires users to manually manage
        /// the buffer life cycle.
        pub unsafe fn ax_alloc_coherent(layout: Layout) -> Option<DMAInfo>;
        /// Deallocates coherent memory previously allocated.
        ///
        /// # Safety
        ///
        /// This function is unsafe because it requires users to manually manage
        /// the buffer life cycle.
        pub unsafe fn ax_dealloc_coherent(dma: DMAInfo, layout: Layout);
    }
}

/// Standard input and output.
pub mod stdio {
    use core::fmt;
    define_api! {
        /// Reads a slice of bytes from the console, returns the number of bytes written.
        pub fn ax_console_read_bytes(buf: &mut [u8]) -> crate::AxResult<usize>;
        /// Writes a slice of bytes to the console, returns the number of bytes written.
        pub fn ax_console_write_bytes(buf: &[u8]) -> crate::AxResult<usize>;
        /// Writes a formatted string to the console.
        pub fn ax_console_write_fmt(args: fmt::Arguments) -> fmt::Result;
    }
}

/// Multi-threading management.
pub mod task {
    define_api_type! {
        @cfg "multitask";
        pub type AxTaskHandle;
        pub type AxWaitQueueHandle;
        pub type AxCpuMask;
    }

    define_api! {
        /// Current task is going to sleep, it will be woken up at the given deadline.
        ///
        /// If the feature `multitask` is not enabled, it uses busy-wait instead
        pub fn ax_sleep_until(deadline: crate::time::AxTimeValue);

        /// Current task gives up the CPU time voluntarily, and switches to another
        /// ready task.
        ///
        /// If the feature `multitask` is not enabled, it does nothing.
        pub fn ax_yield_now();

        /// Exits the current task with the given exit code.
        pub fn ax_exit(exit_code: i32) -> !;
    }

    define_api! {
        @cfg "multitask";

        /// Returns the current task's ID.
        pub fn ax_current_task_id() -> u64;
        /// Spawns a new task with the given entry point and other arguments.
        pub fn ax_spawn(
            f: impl FnOnce() + Send + 'static,
            name: alloc::string::String,
            stack_size: usize
        ) -> AxTaskHandle;
        /// Waits for the given task to exit, and returns its exit code (the
        /// argument of [`ax_exit`]).
        pub fn ax_wait_for_exit(task: AxTaskHandle) -> Option<i32>;
        /// Sets the priority of the current task.
        pub fn ax_set_current_priority(prio: isize) -> crate::AxResult;
        /// Sets the cpu affinity of the current task.
        pub fn ax_set_current_affinity(cpumask: AxCpuMask) -> crate::AxResult;
        /// Blocks the current task and put it into the wait queue, until
        /// other tasks notify the wait queue, or the the given duration has
        /// elapsed (if specified).
        pub fn ax_wait_queue_wait(wq: &AxWaitQueueHandle, timeout: Option<core::time::Duration>) -> bool;
        /// Blocks the current task and put it into the wait queue, until the
        /// given condition becomes true, or the the given duration has elapsed
        /// (if specified).
        pub fn ax_wait_queue_wait_until(
            wq: &AxWaitQueueHandle,
            until_condition: impl Fn() -> bool,
            timeout: Option<core::time::Duration>,
        ) -> bool;
        /// Wakes up one or more tasks in the wait queue.
        ///
        /// The maximum number of tasks to wake up is specified by `count`. If
        /// `count` is `u32::MAX`, it will wake up all tasks in the wait queue.
        pub fn ax_wait_queue_wake(wq: &AxWaitQueueHandle, count: u32);
    }
}

/// Filesystem manipulation operations.
pub mod fs {
    use crate::AxResult;

    define_api_type! {
        @cfg "fs";
        pub type AxFileHandle;
        pub type AxDirHandle;
        pub type AxOpenOptions;
        pub type AxFileAttr;
        pub type AxFileType;
        pub type AxFilePerm;
        pub type AxDirEntry;
        pub type AxSeekFrom;
        #[cfg(feature = "myfs")]
        pub type AxDisk;
        #[cfg(feature = "myfs")]
        pub type MyFileSystemIf;
    }

    define_api! {
        @cfg "fs";

        /// Opens a file at the path relative to the current directory with the
        /// options specified by `opts`.
        pub fn ax_open_file(path: &str, opts: &AxOpenOptions) -> AxResult<AxFileHandle>;
        /// Opens a directory at the path relative to the current directory with
        /// the options specified by `opts`.
        pub fn ax_open_dir(path: &str, opts: &AxOpenOptions) -> AxResult<AxDirHandle>;

        /// Reads the file at the current position, returns the number of bytes read.
        ///
        /// After the read, the cursor will be advanced by the number of bytes read.
        pub fn ax_read_file(file: &mut AxFileHandle, buf: &mut [u8]) -> AxResult<usize>;
        /// Reads the file at the given position, returns the number of bytes read.
        ///
        /// It does not update the file cursor.
        pub fn ax_read_file_at(file: &AxFileHandle, offset: u64, buf: &mut [u8]) -> AxResult<usize>;
        /// Writes the file at the current position, returns the number of bytes
        /// written.
        ///
        /// After the write, the cursor will be advanced by the number of bytes
        /// written.
        pub fn ax_write_file(file: &mut AxFileHandle, buf: &[u8]) -> AxResult<usize>;
        /// Writes the file at the given position, returns the number of bytes
        /// written.
        ///
        /// It does not update the file cursor.
        pub fn ax_write_file_at(file: &AxFileHandle, offset: u64, buf: &[u8]) -> AxResult<usize>;
        /// Truncates the file to the specified size.
        pub fn ax_truncate_file(file: &AxFileHandle, size: u64) -> AxResult;
        /// Flushes the file, writes all buffered data to the underlying device.
        pub fn ax_flush_file(file: &AxFileHandle) -> AxResult;
        /// Sets the cursor of the file to the specified offset. Returns the new
        /// position after the seek.
        pub fn ax_seek_file(file: &mut AxFileHandle, pos: AxSeekFrom) -> AxResult<u64>;
        /// Returns attributes of the file.
        pub fn ax_file_attr(file: &AxFileHandle) -> AxResult<AxFileAttr>;

        /// Reads directory entries starts from the current position into the
        /// given buffer, returns the number of entries read.
        ///
        /// After the read, the cursor of the directory will be advanced by the
        /// number of entries read.
        pub fn ax_read_dir(dir: &mut AxDirHandle, dirents: &mut [AxDirEntry]) -> AxResult<usize>;
        /// Creates a new, empty directory at the provided path.
        pub fn ax_create_dir(path: &str) -> AxResult;
        /// Removes an empty directory.
        ///
        /// If the directory is not empty, it will return an error.
        pub fn ax_remove_dir(path: &str) -> AxResult;
        /// Removes a file from the filesystem.
        pub fn ax_remove_file(path: &str) -> AxResult;
        /// Rename a file or directory to a new name.
        ///
        /// It will delete the original file if `old` already exists.
        pub fn ax_rename(old: &str, new: &str) -> AxResult;

        /// Returns the current working directory.
        pub fn ax_current_dir() -> AxResult<alloc::string::String>;
        /// Changes the current working directory to the specified path.
        pub fn ax_set_current_dir(path: &str) -> AxResult;
    }
}

/// Networking primitives for TCP/UDP communication.
pub mod net {
    use crate::{AxResult, io::AxPollState};
    use core::net::{IpAddr, SocketAddr};

    define_api_type! {
        @cfg "net";
        pub type AxTcpSocketHandle;
        pub type AxUdpSocketHandle;
    }

    define_api! {
        @cfg "net";

        // TCP socket

        /// Creates a new TCP socket.
        pub fn ax_tcp_socket() -> AxTcpSocketHandle;
        /// Returns the local address and port of the TCP socket.
        pub fn ax_tcp_socket_addr(socket: &AxTcpSocketHandle) -> AxResult<SocketAddr>;
        /// Returns the remote address and port of the TCP socket.
        pub fn ax_tcp_peer_addr(socket: &AxTcpSocketHandle) -> AxResult<SocketAddr>;
        /// Moves this TCP socket into or out of nonblocking mode.
        pub fn ax_tcp_set_nonblocking(socket: &AxTcpSocketHandle, nonblocking: bool) -> AxResult;

        /// Connects the TCP socket to the given address and port.
        pub fn ax_tcp_connect(handle: &AxTcpSocketHandle, addr: SocketAddr) -> AxResult;
        /// Binds the TCP socket to the given address and port.
        pub fn ax_tcp_bind(socket: &AxTcpSocketHandle, addr: SocketAddr) -> AxResult;
        /// Starts listening on the bound address and port.
        pub fn ax_tcp_listen(socket: &AxTcpSocketHandle, _backlog: usize) -> AxResult;
        /// Accepts a new connection on the TCP socket.
        ///
        /// This function will block the calling thread until a new TCP connection
        /// is established. When established, a new TCP socket is returned.
        pub fn ax_tcp_accept(socket: &AxTcpSocketHandle) -> AxResult<(AxTcpSocketHandle, SocketAddr)>;

        /// Transmits data in the given buffer on the TCP socket.
        pub fn ax_tcp_send(socket: &AxTcpSocketHandle, buf: &[u8]) -> AxResult<usize>;
        /// Receives data on the TCP socket, and stores it in the given buffer.
        /// On success, returns the number of bytes read.
        pub fn ax_tcp_recv(socket: &AxTcpSocketHandle, buf: &mut [u8]) -> AxResult<usize>;
        /// Returns whether the TCP socket is readable or writable.
        pub fn ax_tcp_poll(socket: &AxTcpSocketHandle) -> AxResult<AxPollState>;
        /// Closes the connection on the TCP socket.
        pub fn ax_tcp_shutdown(socket: &AxTcpSocketHandle) -> AxResult;

        // UDP socket

        /// Creates a new UDP socket.
        pub fn ax_udp_socket() -> AxUdpSocketHandle;
        /// Returns the local address and port of the UDP socket.
        pub fn ax_udp_socket_addr(socket: &AxUdpSocketHandle) -> AxResult<SocketAddr>;
        /// Returns the remote address and port of the UDP socket.
        pub fn ax_udp_peer_addr(socket: &AxUdpSocketHandle) -> AxResult<SocketAddr>;
        /// Moves this UDP socket into or out of nonblocking mode.
        pub fn ax_udp_set_nonblocking(socket: &AxUdpSocketHandle, nonblocking: bool) -> AxResult;

        /// Binds the UDP socket to the given address and port.
        pub fn ax_udp_bind(socket: &AxUdpSocketHandle, addr: SocketAddr) -> AxResult;
        /// Receives a single datagram message on the UDP socket.
        pub fn ax_udp_recv_from(socket: &AxUdpSocketHandle, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)>;
        /// Receives a single datagram message on the UDP socket, without
        /// removing it from the queue.
        pub fn ax_udp_peek_from(socket: &AxUdpSocketHandle, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)>;
        /// Sends data on the UDP socket to the given address. On success,
        /// returns the number of bytes written.
        pub fn ax_udp_send_to(socket: &AxUdpSocketHandle, buf: &[u8], addr: SocketAddr) -> AxResult<usize>;

        /// Connects this UDP socket to a remote address, allowing the `send` and
        /// `recv` to be used to send data and also applies filters to only receive
        /// data from the specified address.
        pub fn ax_udp_connect(socket: &AxUdpSocketHandle, addr: SocketAddr) -> AxResult;
        /// Sends data on the UDP socket to the remote address to which it is
        /// connected.
        pub fn ax_udp_send(socket: &AxUdpSocketHandle, buf: &[u8]) -> AxResult<usize>;
        /// Receives a single datagram message on the UDP socket from the remote
        /// address to which it is connected. On success, returns the number of
        /// bytes read.
        pub fn ax_udp_recv(socket: &AxUdpSocketHandle, buf: &mut [u8]) -> AxResult<usize>;
        /// Returns whether the UDP socket is readable or writable.
        pub fn ax_udp_poll(socket: &AxUdpSocketHandle) -> AxResult<AxPollState>;

        // Miscellaneous

        /// Resolves the host name to a list of IP addresses.
        pub fn ax_dns_query(domain_name: &str) -> AxResult<alloc::vec::Vec<IpAddr>>;
        /// Poll the network stack.
        ///
        /// It may receive packets from the NIC and process them, and transmit queued
        /// packets to the NIC.
        pub fn ax_poll_interfaces() -> AxResult;
    }
}

/// Graphics manipulation operations.
pub mod display {
    define_api_type! {
        @cfg "display";
        pub type AxDisplayInfo;
    }

    define_api! {
        @cfg "display";
        /// Gets the framebuffer information.
        pub fn ax_framebuffer_info() -> AxDisplayInfo;
        /// Flushes the framebuffer, i.e. show on the screen.
        pub fn ax_framebuffer_flush();
    }
}

/// Input/output operations.
pub mod io {
    define_api_type! {
        pub type AxPollState;
    }
}

/// Re-exports of ArceOS modules.
///
/// You should prefer to use other APIs rather than these modules. The modules
/// here should only be used if other APIs do not meet your requirements.
pub mod modules {
    pub use axconfig;
    pub use axhal;
    pub use axlog;
    pub use axruntime;
    pub use axsync;

    #[cfg(feature = "alloc")]
    pub use axalloc;
    #[cfg(feature = "display")]
    pub use axdisplay;
    #[cfg(feature = "dma")]
    pub use axdma;
    #[cfg(any(feature = "fs", feature = "net", feature = "display"))]
    pub use axdriver;
    #[cfg(feature = "fs")]
    pub use axfs;
    #[cfg(feature = "paging")]
    pub use axmm;
    #[cfg(feature = "net")]
    pub use axnet;
    #[cfg(feature = "multitask")]
    pub use axtask;
}
