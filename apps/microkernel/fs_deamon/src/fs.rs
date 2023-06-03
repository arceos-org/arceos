use core::sync::atomic::AtomicUsize;
extern crate alloc;

use alloc::collections::BTreeMap;
use axfs::api::{File as RawFile, OpenOptions};
use libax::axerrno::{ax_err, AxError, AxResult};
use libax::io::{File, Read, Seek, SeekFrom, Write};
use libax::scheme::Packet;
use libax::{scheme::Scheme, Mutex, OpenFlags};
use syscall_number::io::{SEEK_CUR, SEEK_END, SEEK_SET};

#[derive(Default)]
struct VfsScheme {
    handles: Mutex<BTreeMap<usize, RawFile>>,
    next_id: AtomicUsize,
}

pub(crate) fn init_fs() {
    axfs::user_init();
}

pub(crate) fn run() {
    let fs = VfsScheme::default();
    let mut channel = File::create(":/file").unwrap();
    libax::println!("TCP deamon started!");
    loop {
        let mut packet: Packet = Packet::default();
        assert_eq!(
            channel.read_data(&mut packet).unwrap(),
            core::mem::size_of::<Packet>()
        );
        fs.handle(&mut packet);
        assert_eq!(
            channel.write_data(&packet).unwrap(),
            core::mem::size_of::<Packet>()
        );
    }
}

impl Scheme for VfsScheme {
    fn open(&self, path: &str, flags: usize, _uid: u32, _gid: u32) -> AxResult<usize> {
        let flags = OpenFlags::from_bits_truncate(flags);
        let handle = OpenOptions::new()
            .append(flags.contains(OpenFlags::APPEND))
            .truncate(flags.contains(OpenFlags::TRUNCATE))
            .create(flags.contains(OpenFlags::CREATE))
            .read(flags.contains(OpenFlags::READ))
            .write(flags.contains(OpenFlags::WRITE))
            .open(path)?;
        let id = self
            .next_id
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.handles.lock().insert(id, handle);
        info!("FS Open: {} -> {}", path, id);
        Ok(id)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        info!("FS read: {}", id);
        self.handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .read(buf)
    }

    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        info!("FS write: {}", id);
        self.handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .write(buf)
    }

    fn close(&self, id: usize) -> AxResult<usize> {
        info!("FS close: {}", id);
        self.handles
            .lock()
            .remove(&id)
            .ok_or(AxError::BadFileDescriptor)
            .map(|_| 0)
    }

    fn fsync(&self, id: usize) -> AxResult<usize> {
        info!("FS sync: {}", id);
        self.handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .flush()?;
        Ok(0)
    }

    fn seek(&self, id: usize, pos: isize, whence: usize) -> AxResult<isize> {
        info!("FS seek: {}", id);
        let seek = match whence {
            SEEK_CUR => SeekFrom::Current(pos as i64),
            SEEK_END => SeekFrom::End(pos as i64),
            SEEK_SET => SeekFrom::Start(pos as u64),
            _ => return ax_err!(InvalidInput),
        };
        self.handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .seek(seek)
            .map(|x| x as isize)
    }

    fn ftruncate(&self, id: usize, len: usize) -> AxResult<usize> {
        info!("FS truncate: {}", id);
        self.handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
            .set_len(len as u64)?;
        Ok(len)
    }

    fn rmdir(&self, path: &str, _uid: u32, _gid: u32) -> AxResult<usize> {
        info!("FS rmdir: {}", path);
        axfs::api::remove_dir(path)?;
        Ok(0)
    }

    fn unlink(&self, path: &str, _uid: u32, _gid: u32) -> AxResult<usize> {
        info!("FS unlink: {}", path);
        axfs::api::remove_file(path)?;
        Ok(0)
    }
}
