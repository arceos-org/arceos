use axdriver_base::{BaseDriverOps, DevError, DevResult, DeviceType};
use axdriver_block::BlockDriverOps;
use rdrive::{Device, driver::block::io};

#[cfg(feature = "virtio-blk")]
mod virtio;

pub struct Block(Device<rdrive::driver::Block>);

impl BaseDriverOps for Block {
    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }
    fn device_name(&self) -> &str {
        self.0.descriptor().name
    }
}

impl BlockDriverOps for Block {
    fn num_blocks(&self) -> u64 {
        self.0.lock().unwrap().num_blocks() as _
    }
    fn block_size(&self) -> usize {
        self.0.lock().unwrap().block_size()
    }
    fn flush(&mut self) -> DevResult {
        self.0
            .lock()
            .unwrap()
            .flush()
            .map_err(maping_io_err_to_dev_err)
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
        self.0
            .lock()
            .unwrap()
            .read_block(block_id as _, buf)
            .map_err(maping_io_err_to_dev_err)
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
        self.0
            .lock()
            .unwrap()
            .write_block(block_id as _, buf)
            .map_err(maping_io_err_to_dev_err)
    }
}

impl From<Device<rdrive::driver::Block>> for Block {
    fn from(base: Device<rdrive::driver::Block>) -> Self {
        Self(base)
    }
}

fn maping_io_err_to_dev_err(err: io::Error) -> DevError {
    match err.kind {
        io::ErrorKind::Other(_error) => DevError::Io,
        io::ErrorKind::NotAvailable => DevError::BadState,
        io::ErrorKind::BrokenPipe => DevError::BadState,
        io::ErrorKind::InvalidParameter { name: _ } => DevError::InvalidParam,
        io::ErrorKind::InvalidData => DevError::InvalidParam,
        io::ErrorKind::TimedOut => DevError::Io,
        io::ErrorKind::Interrupted => DevError::Again,
        io::ErrorKind::Unsupported => DevError::Unsupported,
        io::ErrorKind::OutOfMemory => DevError::NoMemory,
        io::ErrorKind::WriteZero => DevError::InvalidParam,
    }
}

fn maping_dev_err_to_io_err(err: DevError) -> io::Error {
    let kind = match err {
        DevError::Again => io::ErrorKind::Interrupted,
        DevError::AlreadyExists => io::ErrorKind::Other("Already exists".into()),
        DevError::BadState => io::ErrorKind::BrokenPipe,
        DevError::InvalidParam => io::ErrorKind::InvalidData,
        DevError::Io => io::ErrorKind::Other("I/O error".into()),
        DevError::NoMemory => io::ErrorKind::OutOfMemory,
        DevError::ResourceBusy => io::ErrorKind::Other("Resource busy".into()),
        DevError::Unsupported => io::ErrorKind::Unsupported,
    };
    io::Error {
        kind,
        success_pos: 0,
    }
}
