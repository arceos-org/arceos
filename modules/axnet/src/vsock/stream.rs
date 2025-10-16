use alloc::sync::Arc;
use core::task::Context;

use axerrno::{AxError, AxResult, ax_bail, ax_err_type};
use axio::{Buf, BufMut};
use axpoll::{IoEvents, Pollable};
use axsync::Mutex;

use super::connection_manager::*;
use crate::{
    RecvFlags, RecvOptions, SendOptions, Shutdown,
    general::GeneralOptions,
    options::{Configurable, GetSocketOption, SetSocketOption},
    state::*,
    vsock::{VsockTransport, VsockTransportOps, VsockAddr},
};

pub struct VsockStreamTransport {
    conn_id: Mutex<Option<ConnectionId>>,
    connection: Mutex<Option<Arc<Mutex<Connection>>>>,
    state: StateLock,
    general: GeneralOptions,
}

impl VsockStreamTransport {
    pub fn new() -> Self {
        Self {
            conn_id: Mutex::new(None),
            connection: Mutex::new(None),
            state: StateLock::new(State::Idle),
            general: GeneralOptions::new(),
        }
    }

    fn get_connection(&self) -> AxResult<Arc<Mutex<Connection>>> {
        self.connection.lock().clone().ok_or(AxError::NotConnected)
    }
}

impl Configurable for VsockStreamTransport {
    fn get_option_inner(&self, opt: &mut GetSocketOption) -> AxResult<bool> {
        self.general.get_option_inner(opt)
    }

    fn set_option_inner(&self, opt: SetSocketOption) -> AxResult<bool> {
        self.general.set_option_inner(opt)
    }
}

impl VsockTransportOps for VsockStreamTransport {
    fn bind(&self, mut local_addr: VsockAddr) -> AxResult<()> {
        self.state
            .lock(State::Idle)
            .map_err(|_| ax_err_type!(InvalidInput, "already bound"))?
            .transit(State::Idle, || {
                let mut manager = VSOCK_CONN_MANAGER.lock();
                if local_addr.port == 0 {
                    local_addr.port = manager.allocate_port()?;
                }
                let conn_id = ConnectionId::listening(local_addr.port);
                let conn =
                    manager.create_connection(conn_id, local_addr, None, ConnectionState::Idle);

                *self.conn_id.lock() = Some(conn_id);
                *self.connection.lock() = Some(conn);
                trace!("Vsock binding to {:?}", local_addr);
                Ok(())
            })?;
        Ok(())
    }

    fn listen(&self) -> AxResult<()> {
        let guard = self
            .state
            .lock(State::Idle)
            .map_err(|_| ax_err_type!(InvalidInput, "invalid state for listen"))?;

        guard.transit(State::Listening, || {
            let conn = self.get_connection()?;
            let local_addr = conn.lock().local_addr();

            // register in the global listen table
            VSOCK_CONN_MANAGER.lock().listen(local_addr)?;
            crate::device::vsock_listen(local_addr)?;
            // set state
            conn.lock().set_state(ConnectionState::Listening);
            trace!("Vsock listening on {:?}", local_addr);
            Ok(())
        })
    }

    fn accept(&self) -> AxResult<(VsockTransport, VsockAddr)> {
        if self.state.get() != State::Listening {
            ax_bail!(InvalidInput, "not listening");
        }

        let conn = self.get_connection()?;
        let local_port = conn.lock().local_addr().port;

        // wait for connection
        self.general.recv_poller(self).poll(|| {
            let mut manager = VSOCK_CONN_MANAGER.lock();

            if !manager.can_accept(local_port) {
                return Err(AxError::WouldBlock);
            }

            let (conn_id, peer_addr) = manager.accept(local_port)?;
            let conn = manager.get_connection(&conn_id).ok_or(AxError::NotFound)?;

            // create new VsockStreamTransport
            let new_transport = VsockStreamTransport {
                conn_id: Mutex::new(Some(conn_id)),
                connection: Mutex::new(Some(conn)),
                state: StateLock::new(State::Connected),
                general: GeneralOptions::default(),
            };

            Ok((VsockTransport::Stream(new_transport), peer_addr))
        })
    }

    fn connect(&self, peer_addr: VsockAddr) -> AxResult<()> {
        let guard = self.state.lock(State::Idle).map_err(|state| match state {
            State::Idle => unreachable!(),
            State::Listening => ax_err_type!(InvalidInput, "already listening"),
            State::Connecting => ax_err_type!(InProgress),
            State::Connected => ax_err_type!(AlreadyConnected),
            _ => ax_err_type!(AlreadyConnected),
        })?;

        guard.transit(State::Connecting, || {
            let mut manager = VSOCK_CONN_MANAGER.lock();
            let existing_conn = self.connection.lock();

            // get local address
            let local_port = if let Some(conn) = existing_conn.as_ref() {
                let conn_guard = conn.lock();
                match conn_guard.state() {
                    ConnectionState::Idle => {
                        // already bound but not connected, reuse the port
                        conn_guard.local_addr().port
                    }
                    _ => {
                        // should not happen due to state check above
                        ax_bail!(InvalidInput, "already connected or listening");
                    }
                }
            } else {
                manager.allocate_port()?
            };
            drop(existing_conn);

            let local_addr = VsockAddr {
                cid: crate::device::vsock_guest_cid()?,
                port: local_port,
            };

            // create connection
            let conn_id = ConnectionId::new(local_port, peer_addr.cid, peer_addr.port);
            let conn = manager.create_connection(
                conn_id,
                local_addr,
                Some(peer_addr),
                ConnectionState::Connecting,
            );

            *self.conn_id.lock() = Some(conn_id);
            *self.connection.lock() = Some(conn.clone());

            drop(manager);

            // driver connect
            crate::device::vsock_connect(peer_addr.cid, peer_addr.port, local_port)?;
            debug!("Vsock connecting from {} to {:?}", local_port, peer_addr);
            Ok(())
        })?;

        // wait for connection established
        self.general.send_poller(self).poll(|| {
            let conn = self.get_connection()?;
            let state = conn.lock().state();
            match state {
                ConnectionState::Connected => Ok(()),
                ConnectionState::Connecting => Err(AxError::WouldBlock),
                _ => Err(ax_err_type!(ConnectionRefused)),
            }
        })
    }

    fn send(&self, src: &mut impl Buf, _options: SendOptions) -> AxResult<usize> {
        let conn = self.get_connection()?;
        let conn_guard = conn.lock();

        if conn_guard.state() != ConnectionState::Connected {
            return Err(AxError::NotConnected);
        }

        if conn_guard.tx_closed() {
            return Err(AxError::NotConnected);
        }

        let conn_id = self.conn_id.lock().ok_or(AxError::NotConnected)?;
        drop(conn_guard);

        // now virtio-driver only support non-blocking send
        let result = src.consume(|chunk| {
            crate::device::vsock_send(
                conn_id.peer_cid,
                conn_id.peer_port,
                conn_id.local_port,
                chunk,
            )
        });
        conn.lock().add_tx_bytes(result.unwrap_or(0));
        result
    }

    fn recv(&self, dst: &mut impl BufMut, options: RecvOptions) -> AxResult<usize> {
        let conn = self.get_connection()?;

        self.general.recv_poller(self).poll(|| {
            let mut conn_guard = conn.lock();

            if conn_guard.state() != ConnectionState::Connected {
                return Err(AxError::NotConnected);
            }

            if conn_guard.rx_closed() && conn_guard.rx_buffer_used() == 0 {
                return Ok(0); // EOF
            }

            if conn_guard.rx_buffer_used() == 0 {
                return Err(AxError::WouldBlock);
            }

            let count = if options.flags.contains(RecvFlags::PEEK) {
                // Peek mode: not remove data from buffer
                let available = conn_guard.rx_buffer_used();
                let to_read = dst.remaining_mut().min(available);
                let data: alloc::vec::Vec<u8> =
                    conn_guard.rx_iter().take(to_read).copied().collect();
                dst.write(&data)?
            } else {
                // Normal mode: remove data from buffer
                let (left, right) = conn_guard.rx_slices();
                let mut count = dst.write(left)?;

                if count >= left.len() && !right.is_empty() {
                    count += dst.write(right)?;
                }
                conn_guard.advance_rx_read(count);
                count
            };

            if count > 0 {
                trace!(
                    "Recv {} bytes from connection (buffer_remaining={}/{})",
                    count,
                    conn_guard.rx_buffer_used(),
                    VSOCK_RX_BUFFER_SIZE
                );
                Ok(count)
            } else {
                return Err(AxError::WouldBlock);
            }
        })
    }

    fn shutdown(&self, how: Shutdown) -> AxResult<()> {
        let conn = self.get_connection()?;
        let mut conn = conn.lock();

        if how.has_read() {
            conn.set_rx_closed(true);
        }

        if how.has_write() {
            conn.set_tx_closed(true);
        }

        if let Some(conn_id) = *self.conn_id.lock() {
            if conn.state() == ConnectionState::Connected {
                crate::device::vsock_disconnect(
                    conn_id.peer_cid,
                    conn_id.peer_port,
                    conn_id.local_port,
                )?;
            } else if conn.state() == ConnectionState::Listening {
                VSOCK_CONN_MANAGER.lock().unlisten(conn_id.local_port);
            }
        }
        conn.set_state(ConnectionState::Closed);
        Ok(())
    }

    fn local_addr(&self) -> AxResult<Option<VsockAddr>> {
        Ok(self
            .get_connection()
            .ok()
            .map(|conn| conn.lock().local_addr()))
    }

    fn peer_addr(&self) -> AxResult<Option<VsockAddr>> {
        Ok(self
            .get_connection()
            .ok()
            .and_then(|conn| conn.lock().peer_addr()))
    }
}

impl Pollable for VsockStreamTransport {
    fn poll(&self) -> IoEvents {
        let Ok(conn) = self.get_connection() else {
            return IoEvents::empty();
        };

        let conn = conn.lock();
        let mut events = IoEvents::empty();

        match conn.state() {
            ConnectionState::Listening => {
                // if there is a pending connection, set IN
                if let Some(conn_id) = *self.conn_id.lock() {
                    events.set(
                        IoEvents::IN,
                        VSOCK_CONN_MANAGER.lock().can_accept(conn_id.local_port),
                    );
                }
            }
            ConnectionState::Connected => {
                events.set(IoEvents::IN, conn.rx_buffer_used() > 0 || conn.rx_closed());
                events.set(IoEvents::OUT, !conn.tx_closed());
            }
            ConnectionState::Connecting => {
                // if connected, set OUT
                events.set(IoEvents::OUT, conn.state() == ConnectionState::Connected);
            }
            _ => {}
        }
        events.set(IoEvents::RDHUP, conn.rx_closed());
        events
    }

    fn register(&self, context: &mut Context<'_>, events: IoEvents) {
        if let Ok(conn) = self.get_connection() {
            let mut conn = conn.lock();
            match conn.state() {
                ConnectionState::Listening => {
                    if events.contains(IoEvents::IN) {
                        conn.register_accept_poll(context);
                    }
                }
                ConnectionState::Connected => {
                    if events.contains(IoEvents::IN) {
                        conn.register_rx_poll(context);
                    }
                    if events.contains(IoEvents::OUT) {
                        warn!(
                            "VsockStreamTransport: OUT event on connected socket is not supported"
                        );
                    }
                }
                ConnectionState::Connecting => {
                    if events.contains(IoEvents::OUT) {
                        conn.register_connect_poll(context);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Drop for VsockStreamTransport {
    fn drop(&mut self) {
        let _ = self.shutdown(Shutdown::Both);

        if let Some(conn_id) = *self.conn_id.lock() {
            VSOCK_CONN_MANAGER.lock().remove_connection(&conn_id);
        }
    }
}
