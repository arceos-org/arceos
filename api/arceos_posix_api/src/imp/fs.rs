use alloc::sync::Arc;
use core::{
    ffi::{c_char, c_int},
    mem::size_of,
};

use axerrno::{LinuxError, LinuxResult};
use axfs::fops::OpenOptions;
use axio::{PollState, SeekFrom};
use axsync::Mutex;

use super::fd_ops::{FileLike, get_file_like};
use crate::{ctypes, utils::char_ptr_to_str};

pub struct File {
    inner: Mutex<axfs::fops::File>,
}

pub struct Directory {
    inner: Mutex<axfs::fops::Directory>,
}

// ============================================================================
// Linux-style getdents64 implementation (for normal Linux targets)
// ============================================================================

#[cfg(not(feature = "use-hermit-types"))]
#[repr(C, packed)]
struct LinuxDirent64Head {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
}

#[cfg(not(feature = "use-hermit-types"))]
struct DirBuffer<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

#[cfg(not(feature = "use-hermit-types"))]
impl<'a> DirBuffer<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    fn used_len(&self) -> usize {
        self.offset
    }

    fn remaining_space(&self) -> usize {
        self.buf.len().saturating_sub(self.offset)
    }

    fn write_entry(&mut self, d_ino: u64, d_off: i64, d_type: u8, name: &[u8]) -> bool {
        const NAME_OFFSET: usize = size_of::<LinuxDirent64Head>();

        let name_len = name.len().min(255);
        let reclen = (NAME_OFFSET + name_len + 1).next_multiple_of(8);
        if self.remaining_space() < reclen {
            return false;
        }

        unsafe {
            let entry_ptr = self.buf.as_mut_ptr().add(self.offset);
            entry_ptr
                .cast::<LinuxDirent64Head>()
                .write_unaligned(LinuxDirent64Head {
                    d_ino,
                    d_off,
                    d_reclen: reclen as _,
                    d_type,
                });

            let name_ptr = entry_ptr.add(NAME_OFFSET);
            name_ptr.copy_from_nonoverlapping(name.as_ptr(), name_len);
            name_ptr.add(name_len).write(0);
        }

        self.offset += reclen;
        true
    }
}

// ============================================================================
// Hermit-style getdents64 implementation (for hermit targets)
// ============================================================================

#[cfg(feature = "use-hermit-types")]
use core::mem;

#[cfg(feature = "use-hermit-types")]
struct HermitDirBuffer<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

#[cfg(feature = "use-hermit-types")]
impl<'a> HermitDirBuffer<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    fn used_len(&self) -> usize {
        self.offset
    }

    fn remaining_space(&self) -> usize {
        self.buf.len().saturating_sub(self.offset)
    }

    fn write_entry(&mut self, d_ino: u64, d_type: u8, name: &[u8]) -> bool {
        // Hermit dirent64 structure layout:
        // offset 0: d_ino (u64, 8 bytes)
        // offset 8: d_off (i64, 8 bytes)
        // offset 16: d_reclen (u16, 2 bytes)
        // offset 18: d_type (u8, 1 byte)
        // offset 19: d_name (variable-length null-terminated c_char array)
        const NAME_OFFSET: usize = 19;

        let name_len = name.len().min(255);
        // Total size: fixed header (19 bytes) + name + null terminator
        let dirent_len = NAME_OFFSET + name_len + 1;
        // Align to dirent64 struct alignment (8 bytes for u64)
        let reclen = dirent_len.next_multiple_of(mem::align_of::<ctypes::dirent64>());

        if self.remaining_space() < reclen {
            return false;
        }

        unsafe {
            let entry_ptr = self.buf.as_mut_ptr().add(self.offset);

            // Write fixed fields
            let d_ino_ptr = entry_ptr.cast::<u64>();
            d_ino_ptr.write_unaligned(d_ino);

            let d_off_ptr = entry_ptr.add(8).cast::<i64>();
            d_off_ptr.write_unaligned(0); // d_off is not meaningful in Hermit

            let d_reclen_ptr = entry_ptr.add(16).cast::<u16>();
            d_reclen_ptr.write_unaligned(reclen as u16);

            let d_type_ptr = entry_ptr.add(18);
            d_type_ptr.write(d_type);

            // Write d_name (starting at offset 19)
            let name_ptr = entry_ptr.add(NAME_OFFSET);
            name_ptr.copy_from_nonoverlapping(name.as_ptr(), name_len);
            name_ptr.add(name_len).write(0); // null terminator
        }

        self.offset += reclen;
        true
    }
}

// ============================================================================
// Common file type conversion
// ============================================================================

fn file_type_to_d_type(ty: axfs::fops::FileType) -> u8 {
    match ty {
        axfs::fops::FileType::Dir => 4,      // DT_DIR
        axfs::fops::FileType::File => 8,     // DT_REG
        axfs::fops::FileType::SymLink => 10, // DT_LNK
        _ => 0,                              // DT_UNKNOWN
    }
}

impl File {
    fn new(inner: axfs::fops::File) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl Directory {
    fn new(inner: axfs::fops::Directory) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::ENOTDIR)
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().read(buf)?)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().write(buf)?)
    }

    fn stat(&self) -> LinuxResult<ctypes::stat> {
        let metadata = self.inner.lock().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;
        Ok(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

impl FileLike for Directory {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EISDIR)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EISDIR)
    }

    fn stat(&self) -> LinuxResult<ctypes::stat> {
        let st_mode = 0o040755;
        Ok(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: 0,
            st_blocks: 0,
            st_blksize: 512,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: false,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// Convert open flags to [`OpenOptions`].
fn flags_to_options(flags: c_int, _mode: ctypes::mode_t) -> OpenOptions {
    let flags = flags as u32;
    let mut options = OpenOptions::new();
    match flags & 0b11 {
        ctypes::O_RDONLY => options.read(true),
        ctypes::O_WRONLY => options.write(true),
        _ => {
            options.read(true);
            options.write(true);
        }
    };
    if flags & ctypes::O_APPEND != 0 {
        options.append(true);
    }
    if flags & ctypes::O_TRUNC != 0 {
        options.truncate(true);
    }
    if flags & ctypes::O_CREAT != 0 {
        options.create(true);
    }
    if flags & ctypes::O_EXEC != 0 {
        options.create_new(true);
    }
    options
}

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
pub fn sys_open(filename: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    let filename = char_ptr_to_str(filename);
    debug!("sys_open <= {filename:?} {flags:#o} {mode:#o}");
    syscall_body!(sys_open, {
        let options = flags_to_options(flags, mode);
        let filename = filename?;
        if (flags as u32) & ctypes::O_DIRECTORY != 0 {
            let dir = axfs::fops::Directory::open_dir(filename, &options)?;
            Directory::new(dir).add_to_fd_table()
        } else {
            let file = axfs::fops::File::open(filename, &options)?;
            File::new(file).add_to_fd_table()
        }
    })
}

// ============================================================================
// Linux-style sys_getdents64 (standard Linux targets)
// ============================================================================

/// Read directory entries from `fd` into Linux-style linux_dirent64 buffer.
///
/// Reference: Starry OS implementation
/// Return number of bytes written on success.
#[cfg(not(feature = "use-hermit-types"))]
pub unsafe fn sys_getdents64(fd: c_int, buf: *mut u8, len: usize) -> ctypes::ssize_t {
    debug!("sys_getdents64 (Linux) <= {fd} {:#x} {len}", buf as usize);
    syscall_body!(sys_getdents64, {
        if buf.is_null() || len == 0 {
            return Err(LinuxError::EINVAL);
        }

        let dir = Directory::from_fd(fd).map_err(|_| LinuxError::EBADF)?;
        let mut dir = dir.inner.lock();

        let out = unsafe { core::slice::from_raw_parts_mut(buf, len) };
        let mut dir_buf = DirBuffer::new(out);

        let mut entries: [axfs::fops::DirEntry; 16] =
            core::array::from_fn(|_| axfs::fops::DirEntry::default());
        loop {
            let nr = dir.read_dir(&mut entries)?;
            if nr == 0 {
                break;
            }

            for entry in entries.iter().take(nr) {
                let d_type = file_type_to_d_type(entry.entry_type());
                // Linux style: d_ino, d_off both present
                if !dir_buf.write_entry(1, 0, d_type, entry.name_as_bytes()) {
                    return Ok(dir_buf.used_len() as ctypes::ssize_t);
                }
            }
        }

        Ok(dir_buf.used_len() as ctypes::ssize_t)
    })
}

// ============================================================================
// Hermit-style sys_getdents64 (Hermit/BSD-like targets)
// ============================================================================

/// Read directory entries from `fd` into Hermit-style dirent64 buffer.
///
/// Reference: Hermit OS official implementation
/// Parameters:
/// - `fd`: File Descriptor of the directory in question.
/// - `buf`: Memory for the kernel to store the filled `Dirent64` objects including
///   the c-strings with the filenames.
/// - `len`: Size of the memory region described by `buf` in bytes.
///
/// Return:
/// The number of bytes read into `buf` on success. Zero indicates that no more
/// entries remain and the directory's read position needs to be reset using `sys_lseek`.
/// Negative numbers encode errors.
#[cfg(feature = "use-hermit-types")]
pub unsafe fn sys_getdents64(fd: c_int, buf: *mut u8, len: usize) -> ctypes::ssize_t {
    debug!("sys_getdents64 (Hermit) <= {fd} {:#x} {len}", buf as usize);
    syscall_body!(sys_getdents64, {
        // Hermit ABI: null buffer or zero-sized buffer are invalid
        if buf.is_null() || len == 0 {
            return Err(LinuxError::EINVAL);
        }

        // Hermit returns EINVAL for invalid directory objects
        let dir = Directory::from_fd(fd).map_err(|_| LinuxError::EINVAL)?;
        let mut dir = dir.inner.lock();

        let out = unsafe { core::slice::from_raw_parts_mut(buf, len) };
        let mut dir_buf = HermitDirBuffer::new(out);

        let mut entries: [axfs::fops::DirEntry; 16] =
            core::array::from_fn(|_| axfs::fops::DirEntry::default());
        loop {
            let nr = dir.read_dir(&mut entries)?;
            if nr == 0 {
                break;
            }

            for entry in entries.iter().take(nr) {
                let d_type = file_type_to_d_type(entry.entry_type());
                // Hermit style: only d_ino and d_type, d_off is not meaningful
                if !dir_buf.write_entry(1, d_type, entry.name_as_bytes()) {
                    return Ok(dir_buf.used_len() as ctypes::ssize_t);
                }
            }
        }

        Ok(dir_buf.used_len() as ctypes::ssize_t)
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
pub fn sys_lseek(fd: c_int, offset: ctypes::off_t, whence: c_int) -> ctypes::off_t {
    debug!("sys_lseek <= {fd} {offset} {whence}");
    syscall_body!(sys_lseek, {
        let pos = match whence {
            0 => SeekFrom::Start(offset as _),
            1 => SeekFrom::Current(offset as _),
            2 => SeekFrom::End(offset as _),
            _ => return Err(LinuxError::EINVAL),
        };
        let off = File::from_fd(fd)?.inner.lock().seek(pos)?;
        Ok(off)
    })
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_stat(path: *const c_char, buf: *mut ctypes::stat) -> c_int {
    let path = char_ptr_to_str(path);
    debug!("sys_stat <= {:?} {:#x}", path, buf as usize);
    syscall_body!(sys_stat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let mut options = OpenOptions::new();
        options.read(true);
        let file = axfs::fops::File::open(path?, &options)?;
        let st = File::new(file).stat()?;
        unsafe { *buf = st };
        Ok(0)
    })
}

/// Get file metadata by `fd` and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_fstat(fd: c_int, buf: *mut ctypes::stat) -> c_int {
    debug!("sys_fstat <= {} {:#x}", fd, buf as usize);
    syscall_body!(sys_fstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }

        unsafe { *buf = get_file_like(fd)?.stat()? };
        Ok(0)
    })
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_lstat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_str(path);
    debug!("sys_lstat <= {:?} {:#x}", path, buf as usize);
    syscall_body!(sys_lstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        // ArceOS currently doesn't support symbolic links, so lstat behaves the same as stat
        let mut options = OpenOptions::new();
        options.read(true);
        let file = axfs::fops::File::open(path?, &options)?;
        let st = File::new(file).stat()?;
        unsafe { *buf = st };
        Ok(0)
    })
}

/// Get the path of the current directory.
#[allow(clippy::unnecessary_cast)] // `c_char` is either `i8` or `u8`
pub fn sys_getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    debug!("sys_getcwd <= {:#x} {}", buf as usize, size);
    syscall_body!(sys_getcwd, {
        if buf.is_null() {
            return Ok(core::ptr::null::<c_char>() as _);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size as _) };
        let cwd = axfs::api::current_dir()?;
        let cwd = cwd.as_bytes();
        if cwd.len() < size {
            dst[..cwd.len()].copy_from_slice(cwd);
            dst[cwd.len()] = 0;
            Ok(buf)
        } else {
            Err(LinuxError::ERANGE)
        }
    })
}

/// Rename `old` to `new`
/// If new exists, it is first removed.
///
/// Return 0 if the operation succeeds, otherwise return -1.
pub fn sys_rename(old: *const c_char, new: *const c_char) -> c_int {
    syscall_body!(sys_rename, {
        let old_path = char_ptr_to_str(old)?;
        let new_path = char_ptr_to_str(new)?;
        debug!("sys_rename <= old: {old_path:?}, new: {new_path:?}");
        axfs::api::rename(old_path, new_path)?;
        Ok(0)
    })
}
