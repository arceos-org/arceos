use core::sync::atomic::AtomicUsize;
extern crate alloc;

use alloc::{collections::BTreeMap, sync::Arc};
use axdriver::{prelude::NetDriverOps, AllDevices, AxNetDevice};
use axerrno::{ax_err, AxError, AxResult};
use axsync::Mutex;
use driver_net::{DevError, NetBufferPool};
use scheme::Scheme;

pub struct DeviceScheme {
    net: Option<Arc<NetDevice>>,
    handles: Mutex<BTreeMap<usize, Arc<dyn Device + Sync + Send>>>,
    next_id: AtomicUsize,
}
trait Device {
    fn open(&self, path: &str, id: usize) -> AxResult;
    fn close(&self, id: usize) -> AxResult<usize>;
    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize>;
    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize>;
}

fn init(mut all_device: AllDevices) {
    #[cfg(feature = "user_net")]
    let net = {
        Some(Arc::new({
            let pool = NetBufferPool::new(128, 1526).unwrap();
            let mut net_dev = all_device.net.take_one().expect("No NIC found");
            net_dev.fill_rx_buffers(&pool);
            NetDevice {
                handles: Mutex::new(BTreeMap::new()),
                driver: Mutex::new(net_dev),
                pool,
            }
        }))
    };

    let device_scheme = Arc::new(DeviceScheme {
        net,
        handles: Mutex::new(BTreeMap::new()),
        next_id: 0.into(),
    });
}

impl Scheme for DeviceScheme {
    fn open(&self, path: &str, _flags: usize, _uid: u32, _gid: u32) -> AxResult<usize> {
        let path = path.trim_matches('/').splitn(2, '/');
        let (device, path) = match (path.next(), path.next()) {
            (Some(device), Some(path)) => (device, path),
            (Some(device), None) => (device, ""),
            _ => return ax_err!(NotFound),
        };
        let device: Arc<dyn Device + Sync + Send> = match device {
            "net" => self.net.ok_or(AxError::NotFound)?.clone(),
            _ => return ax_err!(NotFound),
        };
        let id = self
            .next_id
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        device.open(path, id);
        self.handles.lock().insert(id, device);
        Ok(id)
    }
    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        self.handles
            .lock()
            .get(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .read(id, buf)
    }
    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        self.handles
            .lock()
            .get(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .write(id, buf)
    }
    fn close(&self, id: usize) -> AxResult<usize> {
        self.handles
            .lock()
            .remove(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .close(id)
    }
}

enum NetFileType {
    Net,
    Stat { offset: usize },
}
struct NetDevice {
    handles: Mutex<BTreeMap<usize, NetFileType>>,
    driver: Mutex<AxNetDevice>,
    pool: NetBufferPool,
}
impl Device for NetDevice {
    fn open(&self, path: &str, id: usize) -> AxResult {
        match path.trim_matches('/') {
            "" => {
                self.handles.lock().insert(id, NetFileType::Net);
                Ok(())
            }
            "addr" => {
                self.handles
                    .lock()
                    .insert(id, NetFileType::Stat { offset: 0 });
                Ok(())
            }
            _ => ax_err!(NotFound),
        }
    }
    fn close(&self, id: usize) -> AxResult<usize> {
        self.handles
            .lock()
            .remove(&id)
            .map(|_| 0)
            .ok_or(AxError::BadFileDescriptor)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        let handle = self.handles.lock();
        let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;
        match tp {
            NetFileType::Net => {
                // TODO: error type
                let driver = self.driver.lock();
                let rx_buf = driver.receive().map_err(map_err)?;
                let len = rx_buf.packet().len();
                if buf.len() < len {
                    ax_err!(InvalidInput)?;
                }
                // We simply drop rest of the packet
                buf[..len].copy_from_slice(rx_buf.packet());
                driver.recycle_rx_buffer(rx_buf).map_err(map_err)?;
                Ok(len)
            }
            NetFileType::Stat { offset } => {
                let addr = self.driver.lock().mac_address();
                if *offset >= addr.0.len() {
                    Ok(0)
                } else {
                    let write_len = (addr.0.len() - *offset).min(buf.len());
                    buf[0..write_len].copy_from_slice(&addr.0[*offset..*offset + write_len]);
                    *offset += write_len;
                    Ok(write_len)
                }
            }
        }
    }
    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        let handle = self.handles.lock();
        let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;
        match tp {
            NetFileType::Net => {
                let driver = self.driver.lock();
                assert!(driver.can_transmit());

                let tx_buf = self.pool.alloc_boxed().ok_or(AxError::NoMemory)?;
                driver
                    .prepare_tx_buffer(&mut tx_buf, buf.len())
                    .map_err(map_err)?;
                tx_buf.packet_mut().copy_from_slice(buf);
                driver.transmit(&tx_buf).map_err(map_err)?;
                Ok(buf.len())
            }
            NetFileType::Stat { .. } => {
                ax_err!(PermissionDenied)
            }
        }
    }
}

fn map_err(e: DevError) -> AxError {
    match e {
        DevError::Again => AxError::Again,
        DevError::AlreadyExists => AxError::AlreadyExists,
        DevError::BadState => AxError::BadState,
        DevError::InvalidParam => AxError::InvalidInput,
        DevError::Io => AxError::Io,
        DevError::NoMemory => AxError::NoMemory,
        DevError::ResourceBusy => AxError::ResourceBusy,
        DevError::Unsupported => AxError::Unsupported,
    }
}
