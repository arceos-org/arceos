use alloc::boxed::Box;
use alloc::string::String;
use axerrno::AxError;
use axfs::api::ReadDir;
use axfs::fops::File;
use axfs::fops::{FileAttr, FileType, OpenOptions};

#[allow(dead_code)]
pub struct StdDirEntry {
    path: String,
    fname: String,
    ftype: FileType,
}

impl StdDirEntry {
    fn new(path: String, fname: String, ftype: FileType) -> Self {
        Self { path, fname, ftype }
    }
}

#[no_mangle]
pub fn sys_read_dir(path: &str) -> usize {
    let rd = axfs::api::read_dir(path).unwrap();
    let ptr = Box::leak(Box::new(rd));
    ptr as *mut ReadDir as usize
}

#[no_mangle]
pub unsafe fn sys_read_dir_next(handle: usize) -> Option<Result<StdDirEntry, AxError>> {
    let ptr = handle as *mut ReadDir;
    if let Some(Ok(ref de)) = ptr.as_mut().unwrap().next() {
        return Some(Ok(StdDirEntry::new(
            de.path(),
            de.file_name(),
            de.file_type(),
        )));
    }
    None
}

#[no_mangle]
pub fn sys_stat(path: &str) -> Result<FileAttr, AxError> {
    let mut opt = OpenOptions::new();
    opt.read(true);
    File::open(path, &opt)?.get_attr()
}

#[no_mangle]
pub fn sys_open(path: &str, flags: u32) -> Result<usize, AxError> {
    const F_READ: u32 = 0x01;
    const F_WRITE: u32 = 0x02;
    const F_APPEND: u32 = 0x04;
    const F_TRUNC: u32 = 0x08;
    const F_CREATE: u32 = 0x10;
    const F_NEW: u32 = 0x20; /* for create_new */

    axlog::info!("sys_open... {} {:X}", path, flags);
    let mut opts = OpenOptions::new();
    opts.read(flags & F_READ != 0);
    opts.write(flags & F_WRITE != 0);
    opts.append(flags & F_APPEND != 0);
    opts.truncate(flags & F_TRUNC != 0);
    opts.create(flags & F_CREATE != 0);
    opts.create_new(flags & F_NEW != 0);

    axlog::info!("sys_open opts {:?}", opts);
    let f = File::open(path, &opts)?;
    let ptr = Box::leak(Box::new(f));
    Ok(ptr as *mut File as usize)
}

#[no_mangle]
pub fn sys_write(handle: usize, buf: &[u8]) -> usize {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().write(buf).unwrap() }
}

#[no_mangle]
pub fn sys_read(handle: usize, buf: &mut [u8]) -> usize {
    let f = handle as *mut File;
    unsafe { f.as_mut().unwrap().read(buf).unwrap() }
}

#[no_mangle]
pub fn sys_mkdir(path: &str) -> Result<(), AxError> {
    axfs::api::create_dir(path)
}

#[no_mangle]
pub fn sys_rmdir(path: &str) -> Result<(), AxError> {
    axfs::api::remove_dir(path)
}

#[no_mangle]
pub fn sys_unlink(path: &str) -> Result<(), AxError> {
    axfs::api::remove_file(path)
}

#[no_mangle]
pub fn sys_getcwd() -> Result<String, AxError> {
    axfs::api::current_dir()
}

#[no_mangle]
pub fn sys_chdir(path: &str) -> Result<(), AxError> {
    axfs::api::set_current_dir(path)
}

#[no_mangle]
pub fn sys_close_file(handle: usize) {
    unsafe { core::ptr::drop_in_place(handle as *mut File) }
}

#[no_mangle]
pub fn sys_close_dir(handle: usize) {
    unsafe { core::ptr::drop_in_place(handle as *mut ReadDir) }
}
