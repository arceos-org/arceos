use alloc::string::String;
use axerrno::AxResult;
use axfs::fops::{Directory, File};

pub use axfs::fops::DirEntry as AxDirEntry;
pub use axfs::fops::FileAttr as AxFileAttr;
pub use axfs::fops::FilePerm as AxFilePerm;
pub use axfs::fops::FileType as AxFileType;
pub use axfs::fops::OpenOptions as AxOpenOptions;
pub use axio::SeekFrom as AxSeekFrom;

#[cfg(feature = "myfs")]
pub use axfs::fops::{Disk as AxDisk, MyFileSystemIf};

/// A handle to an opened file.
pub struct AxFileHandle(File);

/// A handle to an opened directory.
pub struct AxDirHandle(Directory);

pub fn ax_open_file(path: &str, opts: &AxOpenOptions) -> AxResult<AxFileHandle> {
    Ok(AxFileHandle(File::open(path, opts)?))
}

pub fn ax_open_dir(path: &str, opts: &AxOpenOptions) -> AxResult<AxDirHandle> {
    Ok(AxDirHandle(Directory::open_dir(path, opts)?))
}

pub fn ax_read_file(file: &mut AxFileHandle, buf: &mut [u8]) -> AxResult<usize> {
    file.0.read(buf)
}

pub fn ax_read_file_at(file: &AxFileHandle, offset: u64, buf: &mut [u8]) -> AxResult<usize> {
    file.0.read_at(offset, buf)
}

pub fn ax_write_file(file: &mut AxFileHandle, buf: &[u8]) -> AxResult<usize> {
    file.0.write(buf)
}

pub fn ax_write_file_at(file: &AxFileHandle, offset: u64, buf: &[u8]) -> AxResult<usize> {
    file.0.write_at(offset, buf)
}

pub fn ax_truncate_file(file: &AxFileHandle, size: u64) -> AxResult {
    file.0.truncate(size)
}

pub fn ax_flush_file(file: &AxFileHandle) -> AxResult {
    file.0.flush()
}

pub fn ax_seek_file(file: &mut AxFileHandle, pos: AxSeekFrom) -> AxResult<u64> {
    file.0.seek(pos)
}

pub fn ax_file_attr(file: &AxFileHandle) -> AxResult<AxFileAttr> {
    file.0.get_attr()
}

pub fn ax_read_dir(dir: &mut AxDirHandle, dirents: &mut [AxDirEntry]) -> AxResult<usize> {
    dir.0.read_dir(dirents)
}

pub fn ax_create_dir(path: &str) -> AxResult {
    axfs::api::create_dir(path)
}

pub fn ax_remove_dir(path: &str) -> AxResult {
    axfs::api::remove_dir(path)
}

pub fn ax_remove_file(path: &str) -> AxResult {
    axfs::api::remove_file(path)
}

pub fn ax_rename(old: &str, new: &str) -> AxResult {
    axfs::api::rename(old, new)
}

pub fn ax_current_dir() -> AxResult<String> {
    axfs::api::current_dir()
}

pub fn ax_set_current_dir(path: &str) -> AxResult {
    axfs::api::set_current_dir(path)
}
