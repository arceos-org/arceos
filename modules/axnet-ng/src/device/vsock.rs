use alloc::{collections::VecDeque, string::ToString};
use core::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

use axdriver::prelude::*;
use axerrno::{AxError, AxResult, ax_bail};
use axsync::Mutex;
use axtask::future::{block_on, interruptible};

use crate::vsock::connection_manager::VSOCK_CONN_MANAGER;

// we need a global and static only one vsock device
static VSOCK_DEVICE: Mutex<Option<AxVsockDevice>> = Mutex::new(None);
static PENDING_EVENTS: Mutex<VecDeque<VsockDriverEvent>> = Mutex::new(VecDeque::new());

const VSOCK_RX_TMPBUF_SIZE: usize = 0x1000; // 4KiB buffer for vsock receive

/// Registers a vsock device. Only one vsock device can be registered.
pub fn register_vsock_device(dev: AxVsockDevice) -> AxResult {
    let mut guard = VSOCK_DEVICE.lock();
    if guard.is_some() {
        ax_bail!(AlreadyExists, "vsock device already registered");
    }
    *guard = Some(dev);
    drop(guard);
    Ok(())
}

static POLL_REF_COUNT: Mutex<usize> = Mutex::new(0);
static POLL_TASK_RUNNING: AtomicBool = AtomicBool::new(false);
static POLL_FREQUENCY: PollFrequencyController = PollFrequencyController::new();

struct PollFrequencyController {
    consecutive_idle: AtomicU64,
}

impl PollFrequencyController {
    const fn new() -> Self {
        Self {
            consecutive_idle: AtomicU64::new(0),
        }
    }

    fn current_interval(&self) -> Duration {
        let idle = self.consecutive_idle.load(Ordering::Relaxed);
        let interval_us = match idle {
            0..=3 => 100,     //  3 ：100μs
            4..=10 => 500,    // 4-10 ：500μs
            11..=20 => 2_000, // 11-20 ：2ms
            _ => 10_000,      // 20+ ：10ms
        };
        Duration::from_micros(interval_us)
    }

    fn on_event(&self) {
        self.consecutive_idle.store(0, Ordering::Release);
    }

    fn on_idle(&self) {
        self.consecutive_idle.fetch_add(1, Ordering::Relaxed);
    }

    fn stats(&self) -> (u64, u64) {
        let idle = self.consecutive_idle.load(Ordering::Relaxed);
        let interval = self.current_interval().as_micros() as u64;
        (idle, interval)
    }
}

pub fn start_vsock_poll() {
    let mut count = POLL_REF_COUNT.lock();
    *count += 1;
    let new_count = *count;
    debug!("start_vsock_poll: ref_count -> {}", new_count);
    if new_count == 1 {
        if !POLL_TASK_RUNNING.swap(true, Ordering::SeqCst) {
            drop(count);
            debug!("Starting vsock poll task");
            axtask::spawn_with_name(vsock_poll_loop, "vsock-poll".to_string());
        } else {
            warn!("Poll task already running!");
        }
    }
}

pub fn stop_vsock_poll() {
    let mut count = POLL_REF_COUNT.lock();
    if *count == 0 {
        // this should not happen, log a warning
        warn!("stop_vsock_poll called but ref_count already 0");
        return;
    }
    *count -= 1;
    let new_count = *count;
    debug!("stop_vsock_poll: ref_count -> {new_count}");
}

fn vsock_poll_loop() {
    loop {
        let ref_count = *POLL_REF_COUNT.lock();
        if ref_count == 0 {
            POLL_TASK_RUNNING.store(false, Ordering::SeqCst);
            debug!("Vsock poll task exiting (no active connections)");
            break;
        }
        let _ = block_on(interruptible(poll_interfaces_adaptive()));
    }
}

async fn poll_interfaces_adaptive() -> AxResult<()> {
    let has_events = poll_vsock_interfaces()?;

    if has_events {
        POLL_FREQUENCY.on_event();
    } else {
        POLL_FREQUENCY.on_idle();
    }

    let interval = POLL_FREQUENCY.current_interval();

    let (idle_count, interval_us) = POLL_FREQUENCY.stats();
    if idle_count > 0 && idle_count % 10 == 0 {
        trace!("Poll frequency: idle_count={idle_count}, interval={interval_us}μs",);
    }
    axtask::future::sleep(interval).await;
    Ok(())
}

fn poll_vsock_interfaces() -> AxResult<bool> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    let mut event_count = 0;
    let mut buf = alloc::vec![0; VSOCK_RX_TMPBUF_SIZE];

    // Process pending events first
    // Use core::mem::take to atomically move all events out and empty the global queue
    let pending_events = core::mem::take(&mut *PENDING_EVENTS.lock());
    for event in pending_events {
        handle_vsock_event(event, dev, &mut buf);
    }

    loop {
        match dev.poll_event() {
            Ok(None) => break, // no more events
            Ok(Some(event)) => {
                event_count += 1;
                handle_vsock_event(event, dev, &mut buf);
            }
            Err(e) => {
                info!("Failed to poll vsock event: {e:?}");
                break;
            }
        }
    }
    Ok(event_count > 0)
}

fn handle_vsock_event(event: VsockDriverEvent, dev: &mut AxVsockDevice, buf: &mut [u8]) {
    let mut manager = VSOCK_CONN_MANAGER.lock();
    debug!("Handling vsock event: {event:?}");

    match event {
        VsockDriverEvent::ConnectionRequest(conn_id) => {
            if let Err(e) = manager.on_connection_request(conn_id) {
                info!("Connection request failed: {conn_id:?}, error={e:?}");
            }
        }

        VsockDriverEvent::Received(conn_id, len) => {
            let free_space = if let Some(conn) = manager.get_connection(conn_id) {
                conn.lock().rx_buffer_free()
            } else {
                info!("Received data for unknown connection: {conn_id:?}");
                return;
            };

            if free_space == 0 {
                PENDING_EVENTS
                    .lock()
                    .push_back(VsockDriverEvent::Received(conn_id, len));
                return;
            }

            let max_read = core::cmp::min(free_space, buf.len());
            match dev.recv(conn_id, &mut buf[..max_read]) {
                Ok(read_len) => {
                    if let Err(e) = manager.on_data_received(conn_id, &buf[..read_len]) {
                        info!("Failed to handle received data: conn_id={conn_id:?}, error={e:?}",);
                    }
                }
                Err(e) => {
                    info!("Failed to receive vsock data: conn_id={conn_id:?}, error={e:?}",);
                }
            }
        }

        VsockDriverEvent::Disconnected(conn_id) => {
            if let Err(e) = manager.on_disconnected(conn_id) {
                info!("Failed to handle disconnection: {conn_id:?}, error={e:?}",);
            }
        }

        VsockDriverEvent::Connected(conn_id) => {
            if let Err(e) = manager.on_connected(conn_id) {
                info!("Failed to handle connection established: {conn_id:?}, error={e:?}",);
            }
        }

        VsockDriverEvent::CreditUpdate(conn_id) => {
            if let Err(e) = manager.on_credit_update(conn_id) {
                warn!(
                    "Failed to handle credit update: {:?}, error={:?}",
                    conn_id, e
                );
            }
        }

        VsockDriverEvent::Unknown => warn!("Received unknown vsock event"),
    }
}

pub fn vsock_listen(addr: VsockAddr) -> AxResult<()> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    dev.listen(addr.port);
    Ok(())
}

fn map_dev_err(e: DevError) -> AxError {
    match e {
        DevError::AlreadyExists => AxError::AlreadyExists,
        DevError::Again => AxError::WouldBlock,
        DevError::InvalidParam => AxError::InvalidInput,
        DevError::Io => AxError::Io,
        _ => AxError::BadState,
    }
}

pub fn vsock_connect(conn_id: VsockConnId) -> AxResult<()> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    dev.connect(conn_id).map_err(map_dev_err)
}

pub fn vsock_send(conn_id: VsockConnId, buf: &[u8]) -> AxResult<usize> {
    let max_retries = 10; // Tests have shown that no more than two retries will be notified
    for _ in 0..max_retries {
        let result = {
            let mut guard = VSOCK_DEVICE.lock();
            let dev = guard.as_mut().ok_or(AxError::NotFound)?;
            dev.send(conn_id, buf)
        };
        match result {
            Ok(len) => return Ok(len),
            Err(DevError::Again) => {
                let manager = VSOCK_CONN_MANAGER.lock();
                if let Some(conn) = manager.get_connection(conn_id) {
                    drop(manager);
                    conn.lock().wait_for_tx();
                };
            }
            Err(e) => return Err(map_dev_err(e)),
        }
    }
    Err(map_dev_err(DevError::Again))
}

pub fn vsock_disconnect(conn_id: VsockConnId) -> AxResult<()> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    dev.disconnect(conn_id).map_err(map_dev_err)
}

pub fn vsock_guest_cid() -> AxResult<u64> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    Ok(dev.guest_cid())
}
