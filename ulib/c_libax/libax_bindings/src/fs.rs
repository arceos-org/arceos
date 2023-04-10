use core::ffi::{c_char, CStr};

use crate::AxStat;
use alloc::sync::Arc;
use axerrno::LinuxError;
use libax::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

/// get the path of the current directory
///
/// Returns 0 on failure.
/// Use assert! to ensure the buffer have the enough space.
#[no_mangle]
pub unsafe extern "C" fn ax_getcwd(buf: *const c_char, size: usize) -> *const c_char {
    let dst = core::slice::from_raw_parts_mut(buf as *mut u8, size as _);
    match libax::env::current_dir() {
        Ok(path) => {
            let source = path.as_bytes();
            assert!(source.len() < dst.len(), "getcwd buffer too small");
            dst[..source.len()].copy_from_slice(source);
            dst[source.len()] = 0;
            buf
        }
        Err(_err) => 0 as _,
    }
}

const FILE_LIMIT: usize = 256;
const FD_NONE: Option<Arc<Mutex<File>>> = None;

/// File Descriptor Table
static mut FD_TABLE: Mutex<[Option<Arc<Mutex<File>>>; FILE_LIMIT]> =
    Mutex::new([FD_NONE; FILE_LIMIT]);

/// add a new fd
///
/// Add a file into FD_TABLE and return its fd.
pub fn add_new_fd(file: File) -> Option<usize> {
    let mut fd_table = unsafe { FD_TABLE.lock() };
    fd_table
        .iter_mut()
        .enumerate()
        .find(|(_i, file)| *_i >= 3 && file.is_none())
        .map(|(x, fd)| {
            *fd = Some(Arc::new(Mutex::new(file)));
            x
        })
}

/// open a file
///
/// open a file by filename and insert it into FD_TABLE.
/// return its FD_TABLE index. return ENFILE if file table overflow
#[no_mangle]
pub unsafe extern "C" fn ax_open(filename: *const c_char, _flags: i32) -> isize {
    let filename = CStr::from_ptr(filename).to_str().unwrap();

    match libax::fs::File::options()
        .create(true)
        .read(true)
        .write(true)
        .open(filename)
    {
        Ok(file) => add_new_fd(file).map_or(LinuxError::ENFILE.code() as _, |x| x as _),
        Err(_) => -1,
    }
}

/// Close a fd
#[no_mangle]
pub unsafe extern "C" fn ax_close(fd: usize) -> i32 {
    if fd >= FILE_LIMIT {
        return LinuxError::EBADF.code() as _;
    }
    FD_TABLE.lock()[fd] = None;
    0
}

/// seek the position of the file
///
/// return its position after seek
#[no_mangle]
pub unsafe extern "C" fn ax_lseek(fd: usize, offset: isize, whence: usize) -> i32 {
    if fd >= FILE_LIMIT || FD_TABLE.lock()[fd].is_none() {
        return LinuxError::EBADF.code() as _;
    }
    if whence >= 3 {
        return LinuxError::EINVAL.code() as _;
    }

    let pos = match whence {
        0 => SeekFrom::Start(offset as _),
        1 => SeekFrom::Current(offset as _),
        2 => SeekFrom::End(offset as _),
        _ => unreachable!(),
    };
    FD_TABLE.lock()[fd]
        .as_ref()
        .unwrap()
        .lock()
        .seek(pos)
        .map_or(LinuxError::ESPIPE.code() as _, |x| x as _)
}

/// write data through fd
///
/// return write size if success.
#[no_mangle]
pub unsafe extern "C" fn ax_write(fd: usize, buf: *const u8, count: usize) -> isize {
    if fd >= FILE_LIMIT || FD_TABLE.lock()[fd].is_none() {
        return LinuxError::EBADF.code() as _;
    }
    let src = core::slice::from_raw_parts_mut(buf as *mut u8, count as _);
    FD_TABLE.lock()[fd]
        .as_ref()
        .unwrap()
        .lock()
        .write(src)
        .map_or(LinuxError::EIO.code() as _, |x| x as _)
}

/// read data from file by fd
///
/// return read size if success.
#[no_mangle]
pub unsafe extern "C" fn ax_read(fd: usize, buf: *const u8, count: usize) -> isize {
    if fd >= FILE_LIMIT || FD_TABLE.lock()[fd].is_none() {
        return LinuxError::EBADF.code() as _;
    }
    let src = core::slice::from_raw_parts_mut(buf as *mut u8, count as _);
    FD_TABLE.lock()[fd]
        .as_ref()
        .unwrap()
        .lock()
        .read(src)
        .map_or(LinuxError::EIO.code() as _, |x| x as _)
}

/// get file info by path
///
/// return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_stat(path: *const c_char, stat_ptr: usize) -> isize {
    let stat_ref = (stat_ptr as *mut AxStat).as_mut();
    if stat_ref.is_none() {
        return LinuxError::EINVAL.code() as _;
    }
    let stat = stat_ref.unwrap();

    let path = CStr::from_ptr(path).to_str().unwrap();
    match libax::fs::File::open(path) {
        Ok(file) => {
            stat.st_mode = 0;
            stat.st_ino = 13;
            stat.st_nlink = 13;

            stat.st_uid = 1000;
            stat.st_gid = 1000;
            stat.st_size = file.metadata().unwrap().len() as _;
            stat.st_blksize = 512;

            0
        }
        Err(_) => LinuxError::EPERM.code() as _,
    }
}

/// get file info by fd
///
/// return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn ax_fstat(fd: usize, stat_ptr: usize) -> isize {
    let stat_ref = (stat_ptr as *mut AxStat).as_mut();
    if stat_ref.is_none() {
        return LinuxError::EINVAL.code() as _;
    }
    let stat = stat_ref.unwrap();

    if fd >= FILE_LIMIT || FD_TABLE.lock()[fd].is_none() {
        return LinuxError::EBADF.code() as _;
    }
    let file = &FD_TABLE.lock()[fd];

    stat.st_mode = 0;
    stat.st_ino = 13;
    stat.st_nlink = 1;

    stat.st_uid = 1000;
    stat.st_gid = 1000;
    stat.st_size = file.as_ref().unwrap().lock().metadata().unwrap().len() as _;
    stat.st_blksize = 4096;
    stat.st_rdev = 0;
    stat.st_dev = 1;

    0
}
