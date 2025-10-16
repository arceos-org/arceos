use alloc::{collections::BTreeMap, sync::Arc};

use axerrno::{AxError, AxResult, ax_bail};
use axpoll::PollSet;
use axsync::Mutex;
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::*};

use super::VsockAddr;

pub const VSOCK_RX_BUFFER_SIZE: usize = 64 * 1024; // 64KB receive buffer
const VSOCK_ACCEPT_QUEUE_SIZE: usize = 128; // accept queue size

/// connection unique identifier
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConnectionId {
    pub local_port: u32,
    pub peer_cid: u32,
    pub peer_port: u32,
}

impl ConnectionId {
    pub fn new(local_port: u32, peer_cid: u32, peer_port: u32) -> Self {
        Self {
            local_port,
            peer_cid,
            peer_port,
        }
    }

    pub fn listening(local_port: u32) -> Self {
        Self {
            local_port,
            peer_cid: 0,
            peer_port: 0,
        }
    }
}

/// connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Idle,
    Listening,
    Connecting,
    Connected,
    Closed,
}

/// Connection
pub struct Connection {
    state: ConnectionState,
    local_addr: VsockAddr,
    peer_addr: Option<VsockAddr>,

    /// recv buffer read from driver
    rx_producer: HeapProd<u8>,
    rx_consumer: HeapCons<u8>,

    /// Waker lists
    rx_wakers: PollSet,
    connect_wakers: PollSet,

    /// closed flags
    rx_closed: bool,
    tx_closed: bool,

    /// statistics
    rx_bytes: usize, // received bytes count
    tx_bytes: usize,      // sent bytes count
    dropped_bytes: usize, // dropped bytes count
}

impl Connection {
    fn new(
        local_addr: VsockAddr,
        peer_addr: Option<VsockAddr>,
        state: ConnectionState,
    ) -> Self {
        let rb = HeapRb::<u8>::new(VSOCK_RX_BUFFER_SIZE);
        let (rx_producer, rx_consumer) = rb.split();
        Self {
            state,
            local_addr,
            peer_addr,
            rx_producer,
            rx_consumer,
            rx_wakers: PollSet::new(),
            connect_wakers: PollSet::new(),
            rx_closed: false,
            tx_closed: false,
            rx_bytes: 0,
            tx_bytes: 0,
            dropped_bytes: 0,
        }
    }

    /// Register a waker for transmit events
    pub fn register_accept_poll(&mut self, context: &mut core::task::Context<'_>) {
        // found listen queue
        let manager = VSOCK_CONN_MANAGER.lock();
        let queue = manager
            .get_listen_queue(self.local_addr.port)
            .expect("listen queue not found");
        drop(manager);
        queue.lock().register_poll(context);
    }

    /// Register a waker for receive Events
    pub fn register_rx_poll(&mut self, context: &mut core::task::Context<'_>) {
        self.rx_wakers.register(context.waker());
    }

    /// Register a waker for connect Events
    pub fn register_connect_poll(&mut self, _context: &mut core::task::Context<'_>) {
        self.connect_wakers.register(_context.waker());
    }

    /// Get the free space in the receive buffer
    #[inline]
    pub fn rx_buffer_free(&self) -> usize {
        self.rx_producer.vacant_len()
    }

    /// Get the used space in the receive buffer
    #[inline]
    pub fn rx_buffer_used(&self) -> usize {
        self.rx_consumer.occupied_len()
    }

    /// push data into the receive buffer
    pub fn push_rx_data(&mut self, data: &[u8]) -> usize {
        let available = self.rx_buffer_free();
        let to_write = data.len().min(available);

        if to_write > 0 {
            let written = self.rx_producer.push_slice(&data[..to_write]);
            self.rx_bytes += written;

            if written < data.len() {
                let dropped = data.len() - written;
                self.dropped_bytes += dropped;
                info!(
                    "Vsock connection {:?} rx buffer full, dropped {} bytes",
                    (self.local_addr, self.peer_addr),
                    dropped
                );
            }
            written
        } else {
            self.dropped_bytes += data.len();
            info!(
                "Vsock connection {:?} rx buffer full, dropped {} bytes",
                (self.local_addr, self.peer_addr),
                data.len()
            );
            0
        }
    }

    #[inline]
    pub fn rx_slices(&self) -> (&[u8], &[u8]) {
        self.rx_consumer.as_slices()
    }

    #[inline]
    pub fn advance_rx_read(&mut self, count: usize) {
        unsafe {
            self.rx_consumer.advance_read_index(count);
        }
    }

    #[inline]
    pub fn rx_iter(&self) -> impl Iterator<Item = &u8> {
        self.rx_consumer.iter()
    }

    #[inline]
    pub fn add_tx_bytes(&mut self, count: usize) {
        self.tx_bytes += count;
    }

    #[inline]
    pub fn wake_rx(&mut self) {
        self.rx_wakers.wake();
    }

    #[inline]
    pub fn wake_connect(&mut self) {
        self.connect_wakers.wake();
    }

    #[inline]
    pub fn local_addr(&self) -> VsockAddr {
        self.local_addr
    }

    #[inline]
    pub fn peer_addr(&self) -> Option<VsockAddr> {
        self.peer_addr
    }

    #[inline]
    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    #[inline]
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    #[inline]
    pub fn rx_closed(&self) -> bool {
        self.rx_closed
    }

    #[inline]
    pub fn tx_closed(&self) -> bool {
        self.tx_closed
    }

    #[inline]
    pub fn set_rx_closed(&mut self, closed: bool) {
        self.rx_closed = closed;
    }

    #[inline]
    pub fn set_tx_closed(&mut self, closed: bool) {
        self.tx_closed = closed;
    }
}

/// A fixed-size accept queue
pub struct AcceptQueue {
    producer: ringbuf::HeapProd<ConnectionId>,
    consumer: ringbuf::HeapCons<ConnectionId>,
}

impl AcceptQueue {
    pub fn new() -> Self {
        let rb = HeapRb::<ConnectionId>::new(VSOCK_ACCEPT_QUEUE_SIZE);
        let (producer, consumer) = rb.split();
        Self { producer, consumer }
    }

    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    pub fn push(&mut self, conn_id: ConnectionId) -> AxResult<()> {
        match self.producer.try_push(conn_id) {
            Ok(_) => Ok(()),
            Err(_) => ax_bail!(ResourceBusy, "accept queue full"),
        }
    }

    pub fn pop(&mut self) -> Option<ConnectionId> {
        self.consumer.try_pop()
    }
}

/// listen queue
pub struct ListenQueue {
    pub accept_queue: AcceptQueue,
    pub wakers: PollSet,
    pub local_addr: VsockAddr,
}

impl ListenQueue {
    pub fn new(local_addr: VsockAddr) -> Self {
        Self {
            accept_queue: AcceptQueue::new(),
            wakers: PollSet::new(),
            local_addr,
        }
    }

    pub fn wake(&mut self) {
        self.wakers.wake();
    }

    pub fn register_poll(&mut self, context: &mut core::task::Context<'_>) {
        self.wakers.register(context.waker());
    }
}

/// Global connection manager
pub struct VsockConnectionManager {
    connections: BTreeMap<ConnectionId, Arc<Mutex<Connection>>>,
    listen_queues: BTreeMap<u32, Arc<Mutex<ListenQueue>>>,
    next_ephemeral_port: u32,
}

impl VsockConnectionManager {
    const EPHEMERAL_PORT_END: u32 = 0xffff;
    const EPHEMERAL_PORT_START: u32 = 0xc000;

    pub const fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            listen_queues: BTreeMap::new(),
            next_ephemeral_port: Self::EPHEMERAL_PORT_START,
        }
    }

    /// Get listen queue from specified port
    pub fn get_listen_queue(&self, port: u32) -> Option<Arc<Mutex<ListenQueue>>> {
        self.listen_queues.get(&port).cloned()
    }

    /// allocate an ephemeral port
    pub fn allocate_port(&mut self) -> AxResult<u32> {
        let start = self.next_ephemeral_port;
        loop {
            let port = self.next_ephemeral_port;
            self.next_ephemeral_port = if port >= Self::EPHEMERAL_PORT_END {
                Self::EPHEMERAL_PORT_START
            } else {
                port + 1
            };

            // check if port is in use by listen queue
            if !self.listen_queues.contains_key(&port) {
                // check if port is in use by existing connections
                let port_in_use = self.connections.keys().any(|id| id.local_port == port);
                if !port_in_use {
                    return Ok(port);
                }
            }

            if self.next_ephemeral_port == start {
                ax_bail!(AddressInUse, "no available ports");
            }
        }
    }

    /// create a listen queue
    pub fn listen(&mut self, local_addr: VsockAddr) -> AxResult<()> {
        if self.listen_queues.contains_key(&local_addr.port) {
            ax_bail!(AddressInUse, "port already in use");
        }

        let queue = Arc::new(Mutex::new(ListenQueue::new(local_addr)));
        self.listen_queues.insert(local_addr.port, queue);
        Ok(())
    }

    /// stop listening
    pub fn unlisten(&mut self, port: u32) {
        self.listen_queues.remove(&port);
        debug!("Vsock unlisten on port {}", port);
    }

    /// check if port accept
    pub fn can_accept(&self, port: u32) -> bool {
        self.listen_queues
            .get(&port)
            .map(|q| !q.lock().accept_queue.is_empty())
            .unwrap_or(false)
    }

    /// accept a connection
    pub fn accept(&mut self, port: u32) -> AxResult<(ConnectionId, VsockAddr)> {
        let queue = self.listen_queues.get(&port).ok_or(AxError::InvalidInput)?;

        let conn_id = queue.lock().accept_queue.pop().ok_or(AxError::WouldBlock)?;

        let conn = self.connections.get(&conn_id).ok_or(AxError::NotFound)?;

        let peer_addr = conn.lock().peer_addr.ok_or(AxError::NotFound)?;

        debug!("Accepted connection: {:?} from {:?}", conn_id, peer_addr);
        Ok((conn_id, peer_addr))
    }

    /// create a new connection
    pub fn create_connection(
        &mut self,
        conn_id: ConnectionId,
        local_addr: VsockAddr,
        peer_addr: Option<VsockAddr>,
        state: ConnectionState,
    ) -> Arc<Mutex<Connection>> {
        let conn = Connection::new(local_addr, peer_addr, state);
        let conn = Arc::new(Mutex::new(conn));
        if self.connections.contains_key(&conn_id) {
            info!("Connection {:?} already exists, overwriting", conn_id);
        } else {
            crate::device::start_vsock_poll();
        }
        self.connections.insert(conn_id, conn.clone());
        debug!(
            "Created connection {:?}: local={:?}, peer={:?}",
            conn_id, local_addr, peer_addr
        );
        conn
    }

    /// get a connection by id
    pub fn get_connection(&self, conn_id: &ConnectionId) -> Option<Arc<Mutex<Connection>>> {
        self.connections.get(conn_id).cloned()
    }

    /// remove a connection
    pub fn remove_connection(&mut self, conn_id: &ConnectionId) {
        if let Some(conn) = self.connections.remove(conn_id) {
            let conn = conn.lock();
            crate::device::stop_vsock_poll();
            debug!(
                "Removed connection {:?}: rx={} bytes, tx={} bytes, dropped={} bytes",
                conn_id, conn.rx_bytes, conn.tx_bytes, conn.dropped_bytes
            );
        }
    }

    /// handle a new connection request (by driver event)
    pub fn on_connection_request(
        &mut self,
        local_port: u32,
        peer_addr: VsockAddr,
    ) -> AxResult<()> {
        let queue = self
            .listen_queues
            .get(&local_port)
            .ok_or(AxError::NotFound)?
            .clone();

        let conn_id = ConnectionId::new(local_port, peer_addr.cid, peer_addr.port);
        let local_addr = queue.lock().local_addr;

        // check if connection already exists
        if self.connections.contains_key(&conn_id) {
            warn!("Connection {:?} already exists, ignoring request", conn_id);
            return Ok(());
        }

        // create new connection
        self.create_connection(
            conn_id,
            local_addr,
            Some(peer_addr),
            ConnectionState::Connected,
        );

        // 加入 accept 队列
        let mut queue_guard = queue.lock();
        if let Err(_) = queue_guard.accept_queue.push(conn_id) {
            info!(
                "Accept queue full for port {}, dropping connection from {:?}",
                local_port, peer_addr
            );
            // full -- remove the connection
            drop(queue_guard);
            self.remove_connection(&conn_id);
            return Err(AxError::ResourceBusy);
        }

        queue_guard.wake();
        drop(queue_guard);

        trace!(
            "New connection request from {:?} on port {}",
            peer_addr, local_port
        );
        Ok(())
    }

    /// handle data received (by driver event)
    pub fn on_data_received(&mut self, conn_id: &ConnectionId, data: &[u8]) -> AxResult<()> {
        let conn = self
            .connections
            .get(conn_id)
            .ok_or(AxError::NotFound)?
            .clone();

        let mut conn_guard = conn.lock();
        let written = conn_guard.push_rx_data(data);
        if written > 0 {
            conn_guard.wake_rx();
        }

        trace!(
            "Received {} bytes for connection {:?} (written={}, buffer_used={}/{})",
            data.len(),
            conn_id,
            written,
            conn_guard.rx_buffer_used(),
            VSOCK_RX_BUFFER_SIZE
        );
        Ok(())
    }

    /// handle disconnection (by driver event)
    pub fn on_disconnected(&mut self, conn_id: &ConnectionId) -> AxResult<()> {
        if let Some(conn) = self.connections.get(conn_id) {
            let mut conn_guard = conn.lock();
            conn_guard.state = ConnectionState::Closed;
            conn_guard.rx_closed = true;
            conn_guard.tx_closed = true;
            conn_guard.wake_rx();
            trace!("Connection {:?} disconnected", conn_id);
        }
        Ok(())
    }

    /// handle connected event (by driver event)
    pub fn on_connected(&mut self, conn_id: &ConnectionId) -> AxResult<()> {
        if let Some(conn) = self.connections.get(conn_id) {
            let mut conn_guard = conn.lock();
            conn_guard.state = ConnectionState::Connected;
            conn_guard.wake_connect();
            trace!("Connection {:?} established", conn_id);
        }
        Ok(())
    }

    /// statistics
    #[allow(dead_code)]
    pub fn get_stats(&self) -> VsockStats {
        VsockStats {
            total_connections: self.connections.len(),
            listening_ports: self.listen_queues.len(),
            total_rx_bytes: self.connections.values().map(|c| c.lock().rx_bytes).sum(),
            total_tx_bytes: self.connections.values().map(|c| c.lock().tx_bytes).sum(),
            total_dropped_bytes: self
                .connections
                .values()
                .map(|c| c.lock().dropped_bytes)
                .sum(),
        }
    }
}

/// Vsock statistics
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VsockStats {
    pub total_connections: usize,
    pub listening_ports: usize,
    pub total_rx_bytes: usize,
    pub total_tx_bytes: usize,
    pub total_dropped_bytes: usize,
}

pub static VSOCK_CONN_MANAGER: Mutex<VsockConnectionManager> = Mutex::new(VsockConnectionManager::new());

/// for debug
#[allow(dead_code)]
pub fn get_vsock_stats() -> VsockStats {
    VSOCK_CONN_MANAGER.lock().get_stats()
}
