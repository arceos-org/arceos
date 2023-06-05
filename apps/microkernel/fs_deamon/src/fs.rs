use core::sync::atomic::AtomicUsize;
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use axfs::api::{create_dir, read_dir, File as RawFile, OpenOptions};
use libax::axerrno::{ax_err, AxError, AxResult};
use libax::io::{File, Read, Seek, SeekFrom, Write};
use libax::scheme::{Packet, Stat};
use libax::{scheme::Scheme, Mutex, OpenFlags};
use syscall_number::io::{SEEK_CUR, SEEK_END, SEEK_SET};

enum FileHandle {
    Directory {
        path: String,
        offset: usize,
        data: Option<Vec<u8>>,
    },
    File {
        path: String,
        handle: RawFile,
    },
}

#[derive(Default)]
struct VfsScheme {
    handles: Mutex<BTreeMap<usize, FileHandle>>,
    next_id: AtomicUsize,
}

pub(crate) fn init_fs() {
    axfs::user_init();
    libax::println!("FS inited");
}

pub(crate) fn run() {
    let fs = VfsScheme::default();
    let mut channel = File::create(":/file").unwrap();
    libax::println!("FS deamon started!");
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
        let handle = if flags.contains(OpenFlags::DIRECTORY) {
            if flags.contains(OpenFlags::CREATE) {
                match create_dir(path) {
                    Ok(_) => {}
                    Err(AxError::AlreadyExists) if !flags.contains(OpenFlags::EXCL) => {}
                    Err(e) => ax_err!(e)?,
                }
            }
            FileHandle::Directory {
                path: path.into(),
                offset: 0,
                data: None,
            }
        } else {
            FileHandle::File {
                path: path.into(),
                handle: OpenOptions::new()
                    .append(flags.contains(OpenFlags::APPEND))
                    .truncate(flags.contains(OpenFlags::TRUNCATE))
                    .create(flags.contains(OpenFlags::CREATE) && !flags.contains(OpenFlags::EXCL))
                    .create_new(flags.contains(OpenFlags::CREATE | OpenFlags::EXCL))
                    .read(flags.contains(OpenFlags::READ))
                    .write(flags.contains(OpenFlags::WRITE))
                    .open(path)?,
            }
        };
        let id = self
            .next_id
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.handles.lock().insert(id, handle);
        info!("FS Open: {} -> {}", path, id);
        Ok(id)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        info!("FS read: {}", id);
        match self
            .handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
        {
            FileHandle::Directory { path, offset, data } => {
                if data.is_none() {
                    let mut data_inner: Vec<u8> = Vec::new();
                    for item in read_dir(path)? {
                        let item = item?;
                        data_inner.extend_from_slice(item.file_name().as_bytes());
                        data_inner.push(b'\n');
                    }
                    *data = Some(data_inner);
                }
                let data = data.as_mut().unwrap();
                if *offset >= data.len() {
                    return Ok(0);
                }
                let read_len = buf.len().min(data.len() - *offset);
                buf[0..read_len].copy_from_slice(&data[*offset..*offset + read_len]);
                *offset += read_len;
                Ok(read_len)
            }
            FileHandle::File { handle, .. } => handle.read(buf),
        }
    }

    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        info!("FS write: {}", id);
        match self
            .handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
        {
            FileHandle::Directory { .. } => ax_err!(BadFileDescriptor),
            FileHandle::File { handle, .. } => handle.write(buf),
        }
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
        match self
            .handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
        {
            FileHandle::Directory { .. } => ax_err!(BadFileDescriptor),
            FileHandle::File { handle, .. } => handle.flush(),
        }?;
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
        match self
            .handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
        {
            FileHandle::Directory { .. } => ax_err!(BadFileDescriptor),
            FileHandle::File { handle, .. } => handle.seek(seek),
        }
        .map(|x| x as isize)
    }

    fn ftruncate(&self, id: usize, len: usize) -> AxResult<usize> {
        info!("FS truncate: {}", id);
        match self
            .handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
        {
            FileHandle::Directory { .. } => ax_err!(BadFileDescriptor),
            FileHandle::File { handle, .. } => handle.set_len(len as u64),
        }?;
        Ok(len)
    }

    fn fstat(&self, id: usize, stat: &mut Stat) -> AxResult<usize> {
        info!("FS fstat: {}", id);
        *stat = *match self
            .handles
            .lock()
            .get_mut(&id)
            .ok_or(AxError::BadFileDescriptor)?
        {
            FileHandle::Directory { path, .. } => axfs::api::metadata(path),
            FileHandle::File { path, .. } => axfs::api::metadata(path),
        }?
        .raw_metadata();
        Ok(core::mem::size_of_val(stat))
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
