//! Scheme about all driver operations
use core::sync::atomic::AtomicUsize;
extern crate alloc;

use alloc::vec::Vec;
use alloc::{collections::BTreeMap, sync::Arc};
use axdriver::prelude::*;
use axdriver::AllDevices;
use axerrno::{ax_err, AxError, AxResult};
use axsync::Mutex;
use scheme::Scheme;

#[cfg(feature = "user_fs")]
use self::block::BlockDev;
#[cfg(feature = "user_net")]
use self::net::NetDevice;

use super::schemes;
use super::KernelScheme;

/// Device Scheme
pub struct DeviceScheme {
    #[cfg(feature = "user_net")]
    net: Option<Arc<NetDevice>>,
    #[cfg(feature = "user_fs")]
    fs: Option<Arc<BlockDev>>,
    handles: Mutex<BTreeMap<usize, Arc<dyn Device + Sync + Send>>>,
    next_id: AtomicUsize,
}
trait Device {
    fn open(&self, path: &str, id: usize) -> AxResult;
    fn close(&self, id: usize) -> AxResult<usize>;
    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize>;
    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize>;
    #[allow(unused)]
    fn lseek(&self, id: usize, offset: isize, whence: usize) -> AxResult<isize> {
        ax_err!(Unsupported)
    }
}

/// Initializes all devices
#[allow(unused_variables, unused_mut)]
pub fn init(mut all_device: AllDevices) {
    #[cfg(feature = "user_net")]
    let net = self::net::init(&mut all_device);

    #[cfg(feature = "user_fs")]
    let fs = self::block::init(&mut all_device);

    let device_scheme = Arc::new(DeviceScheme {
        #[cfg(feature = "user_net")]
        net,
        #[cfg(feature = "user_fs")]
        fs,
        handles: Mutex::new(BTreeMap::new()),
        next_id: 0.into(),
    });

    schemes().insert("dev", device_scheme);
}

impl Scheme for DeviceScheme {
    #[allow(unreachable_code, unused_variables)]
    fn open(&self, path: &str, _flags: usize, _uid: u32, _gid: u32) -> AxResult<usize> {
        let mut path = path.trim_matches('/').splitn(2, '/');
        let (device, path) = match (path.next(), path.next()) {
            (Some(device), Some(path)) => (device, path),
            (Some(device), None) => (device, ""),
            _ => return ax_err!(NotFound),
        };

        let device: Arc<dyn Device + Sync + Send> = match device {
            #[cfg(feature = "user_net")]
            "net" => self.net.clone().ok_or(AxError::NotFound)?,
            #[cfg(feature = "user_fs")]
            "disk" => self.fs.clone().ok_or(AxError::NotFound)?,
            _ => return ax_err!(NotFound),
        };
        let id = self
            .next_id
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        device.open(path, id)?;
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
    fn seek(&self, id: usize, pos: isize, whence: usize) -> AxResult<isize> {
        self.handles
            .lock()
            .get(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .lseek(id, pos, whence)
    }
}

impl KernelScheme for DeviceScheme {}

#[cfg(feature = "user_net")]
mod net {
    extern crate alloc;
    use alloc::{collections::BTreeMap, sync::Arc};
    use axdriver::{AllDevices, AxNetDevice};
    use axerrno::{ax_err, AxError, AxResult};
    use axsync::Mutex;
    use driver_net::{NetBufferPool, NetDriverOps};
    use lazy_init::LazyInit;

    use super::{map_err, Device, ReadOnlyFile};

    enum NetFileType {
        Net,
        Stat(ReadOnlyFile),
    }
    pub struct NetDevice {
        handles: Mutex<BTreeMap<usize, NetFileType>>,
        driver: Mutex<AxNetDevice>,
    }
    static POOL: LazyInit<NetBufferPool> = LazyInit::new();
    pub fn init(all_device: &mut AllDevices) -> Option<Arc<NetDevice>> {
        info!("dev:/net started");
        Some(Arc::new({
            let pool = NetBufferPool::new(128, 1526).unwrap();
            POOL.init_by(pool);

            let mut net_dev = all_device.net.take_one().expect("No NIC found");
            net_dev.fill_rx_buffers(&POOL).unwrap();
            NetDevice {
                handles: Mutex::new(BTreeMap::new()),
                driver: Mutex::new(net_dev),
            }
        }))
    }
    impl Device for NetDevice {
        fn open(&self, path: &str, id: usize) -> AxResult {
            match path.trim_matches('/') {
                "" => {
                    self.handles.lock().insert(id, NetFileType::Net);
                    Ok(())
                }
                "addr" => {
                    self.handles.lock().insert(
                        id,
                        NetFileType::Stat(ReadOnlyFile::new(&self.driver.lock().mac_address().0)),
                    );
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
            trace!("Net read {}", id);
            let mut handle = self.handles.lock();
            let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;
            match tp {
                NetFileType::Net => {
                    // TODO: error type
                    let mut driver = self.driver.lock();
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
                NetFileType::Stat(file) => Ok(file.read(buf)),
            }
        }
        fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
            trace!("Net write {}", id);
            let mut handle = self.handles.lock();
            let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;
            match tp {
                NetFileType::Net => {
                    let mut driver = self.driver.lock();
                    assert!(driver.can_transmit());

                    let mut tx_buf = POOL.alloc_boxed().ok_or(AxError::NoMemory)?;
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
}

#[cfg(feature = "user_fs")]
mod block {
    use alloc::{collections::BTreeMap, sync::Arc};
    use axdriver::{AllDevices, AxBlockDevice};
    use axerrno::{ax_err, AxError, AxResult};
    use axsync::Mutex;
    use driver_block::{BaseDriverOps, BlockDriverOps};
    use syscall_number::io::{SEEK_CUR, SEEK_END, SEEK_SET};

    use crate::dev::map_err;

    use super::{Device, ReadOnlyFile};

    enum BlockFileType {
        Block { offset: usize },
        Stat(ReadOnlyFile),
    }
    pub struct BlockDev {
        handles: Mutex<BTreeMap<usize, BlockFileType>>,
        driver: Mutex<AxBlockDevice>,
    }

    pub fn init(all_device: &mut AllDevices) -> Option<Arc<BlockDev>> {
        info!("dev:/block started");
        Some(Arc::new({
            let dev = all_device.block.take_one().expect("No block device found!");
            info!("  use block device 0: {:?}", dev.device_name());
            BlockDev {
                handles: Mutex::new(BTreeMap::new()),
                driver: Mutex::new(dev),
            }
        }))
    }

    impl Device for BlockDev {
        fn open(&self, path: &str, id: usize) -> AxResult {
            match path.trim_matches('/') {
                "" => {
                    self.handles
                        .lock()
                        .insert(id, BlockFileType::Block { offset: 0 });
                    Ok(())
                }
                "num_blocks" => {
                    self.handles.lock().insert(
                        id,
                        BlockFileType::Stat(ReadOnlyFile::new(
                            &self.driver.lock().num_blocks().to_ne_bytes(),
                        )),
                    );
                    Ok(())
                }
                "block_size" => {
                    self.handles.lock().insert(
                        id,
                        BlockFileType::Stat(ReadOnlyFile::new(
                            &self.driver.lock().block_size().to_ne_bytes(),
                        )),
                    );
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
            trace!("Block read {}", id);
            let mut handle = self.handles.lock();
            let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;
            match tp {
                BlockFileType::Block { offset } => {
                    let mut driver = self.driver.lock();
                    let block_size = driver.block_size();
                    assert!(*offset % block_size == 0);
                    if buf.len() % block_size != 0 {
                        return ax_err!(InvalidInput);
                    }
                    driver
                        .read_block((*offset / block_size) as u64, buf)
                        .map_err(map_err)?;
                    *offset += buf.len();
                    Ok(buf.len())
                }
                BlockFileType::Stat(file) => Ok(file.read(buf)),
            }
        }

        fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
            trace!("Block read {}", id);
            let mut handle = self.handles.lock();
            let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;

            match tp {
                BlockFileType::Block { offset } => {
                    // To make it really a block device,
                    // we only accept aligned reads and writes
                    let mut driver = self.driver.lock();
                    let block_size = driver.block_size();
                    assert!(*offset % block_size == 0);
                    if buf.len() % block_size != 0 {
                        return ax_err!(InvalidInput);
                    }
                    driver
                        .write_block((*offset / block_size) as u64, buf)
                        .map_err(map_err)?;
                    *offset += buf.len();
                    Ok(buf.len())
                }
                BlockFileType::Stat(_) => ax_err!(PermissionDenied),
            }
        }

        fn lseek(&self, id: usize, offset: isize, whence: usize) -> AxResult<isize> {
            trace!("Block seek {}", id);
            let mut handle = self.handles.lock();
            let tp = handle.get_mut(&id).ok_or(AxError::BadFileDescriptor)?;

            match tp {
                BlockFileType::Block { offset: old_offset } => {
                    let new_offset = match whence {
                        SEEK_SET => {
                            if offset >= 0 {
                                offset as usize
                            } else {
                                return ax_err!(InvalidInput, "offset < 0");
                            }
                        }
                        SEEK_CUR => *old_offset + offset as usize,
                        SEEK_END => {
                            (self.driver.lock().num_blocks() as usize
                                * self.driver.lock().block_size())
                                + (offset as usize)
                        }
                        _ => return ax_err!(InvalidInput, "whence"),
                    };
                    let block_size = self.driver.lock().block_size();
                    if new_offset % block_size != 0 {
                        return ax_err!(InvalidInput, "offset not aligned");
                    }

                    *old_offset = new_offset;
                    Ok(new_offset as isize)
                }

                BlockFileType::Stat(_) => ax_err!(PermissionDenied),
            }
        }
    }
}

struct ReadOnlyFile {
    data: Vec<u8>,
    offset: usize,
}

impl ReadOnlyFile {
    fn new(data: &[u8]) -> ReadOnlyFile {
        ReadOnlyFile {
            data: Vec::from(data),
            offset: 0,
        }
    }
    fn read(&mut self, buf: &mut [u8]) -> usize {
        if self.offset >= self.data.len() {
            return 0;
        }
        let read_len = buf.len().min(self.data.len() - self.offset);
        buf.copy_from_slice(&self.data[self.offset..self.offset + read_len]);
        self.offset += read_len;
        read_len
    }
}

#[allow(dead_code)]
fn map_err(e: DevError) -> AxError {
    match e {
        DevError::Again => AxError::WouldBlock,
        DevError::AlreadyExists => AxError::AlreadyExists,
        DevError::BadState => AxError::BadState,
        DevError::InvalidParam => AxError::InvalidInput,
        DevError::Io => AxError::Io,
        DevError::NoMemory => AxError::NoMemory,
        DevError::ResourceBusy => AxError::ResourceBusy,
        DevError::Unsupported => AxError::Unsupported,
    }
}
