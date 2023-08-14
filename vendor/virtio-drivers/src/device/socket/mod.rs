//! This module implements the virtio vsock device.

mod error;
mod protocol;
mod vsock;

pub use error::SocketError;
pub use vsock::{DisconnectReason, VirtIOSocket, VsockEvent, VsockEventType};
