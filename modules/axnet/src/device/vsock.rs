use core::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

use axdriver::prelude::*;
use axerrno::{AxError, AxResult, LinuxError, ax_bail};
use axsync::Mutex;
use axtask::future::{block_on, interruptible};

use crate::{
    alloc::string::ToString,
    vsock::{
        VsockAddr,
        connection_manager::{ConnectionId, VSOCK_CONN_MANAGER},
    },
};

// we need a global and static only one vsock device
static VSOCK_DEVICE: Mutex<Option<AxVsockDevice>> = Mutex::new(None);

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
            axtask::spawn(vsock_poll_loop, "vsock-poll".to_string());
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
    debug!("stop_vsock_poll: ref_count -> {}", new_count);
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
        trace!(
            "Poll frequency: idle_count={}, interval={}μs",
            idle_count, interval_us
        );
    }
    axtask::future::sleep(interval).await;
    Ok(())
}

fn poll_vsock_interfaces() -> AxResult<bool> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    let mut event_count = 0;

    loop {
        match dev.poll_event() {
            Ok(None) => break, // no more events
            Ok(Some(event)) => {
                event_count += 1;
                handle_vsock_event(event);
            }
            Err(e) => {
                info!("Failed to poll vsock event: {:?}", e);
                break;
            }
        }
    }
    Ok(event_count > 0)
}

fn handle_vsock_event(event: VsockDriverEvent) {
    let mut manager = VSOCK_CONN_MANAGER.lock();
    debug!("Handling vsock event: {:?}", event);

    match event {
        VsockDriverEvent::ConnectionRequest(local_port, peer_cid, peer_port) => {
            let peer_addr = VsockAddr {
                cid: peer_cid,
                port: peer_port,
            };
            let _ = manager.on_connection_request(local_port, peer_addr);
        }

        VsockDriverEvent::DataReceived(local_port, peer_cid, peer_port, data) => {
            let conn_id = ConnectionId::new(local_port, peer_cid, peer_port);
            let _ = manager.on_data_received(&conn_id, &data);
        }

        VsockDriverEvent::Disconnect(local_port, peer_cid, peer_port) => {
            let conn_id = ConnectionId::new(local_port, peer_cid, peer_port);
            let _ = manager.on_disconnected(&conn_id);
        }

        VsockDriverEvent::Connected(local_port, peer_cid, peer_port) => {
            let conn_id = ConnectionId::new(local_port, peer_cid, peer_port);
            let _ = manager.on_connected(&conn_id);
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
        DevError::Again => AxError::Other(LinuxError::EAGAIN),
        DevError::InvalidParam => AxError::InvalidInput,
        DevError::Io => AxError::Io,
        _ => AxError::BadState,
    }
}

pub fn vsock_connect(peer_cid: u32, peer_port: u32, src_port: u32) -> AxResult<()> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    dev.connect(peer_cid, peer_port, src_port)
        .map_err(map_dev_err)
}

pub fn vsock_send(peer_cid: u32, peer_port: u32, src_port: u32, buf: &[u8]) -> AxResult<usize> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    dev.send(peer_cid, peer_port, src_port, buf)
        .map_err(map_dev_err)
}

pub fn vsock_disconnect(peer_cid: u32, peer_port: u32, src_port: u32) -> AxResult<()> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    dev.disconnect(peer_cid, peer_port, src_port)
        .map_err(map_dev_err)
}

pub fn vsock_guest_cid() -> AxResult<u32> {
    let mut guard = VSOCK_DEVICE.lock();
    let dev = guard.as_mut().ok_or(AxError::NotFound)?;
    Ok(dev.guest_cid())
}
