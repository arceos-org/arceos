//! Schemes implementations in kernel mode
//! see: <https://gitlab.redox-os.org/redox-os/syscall/-/blob/master/src/scheme/scheme.rs>
//!
//! # Features
//!
//! `user_fs`: block drivers
//! `user_net`: net drivers
//! `process`: process support
#![no_std]

extern crate alloc;

#[macro_use]
extern crate axlog;

#[macro_use]
extern crate crate_interface;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{collections::BTreeMap, sync::Arc};
use axerrno::{ax_err, to_ret_code, AxError, AxResult};
use axmem::{copy_byte_buffer_to_user, copy_slice_from_user};
use axsync::{Mutex, MutexGuard};
use axtask::current;
use lazy_init::LazyInit;

/// File handle
pub struct FileHandle {
    /// related scheme id
    pub scheme_id: SchemeId,
    /// id in the scheme
    pub file_id: usize,
}

/// Gets current file table
#[crate_interface::def_interface]
pub trait CurrentFileTable {
    /// Get current file table
    fn current_file_table() -> Arc<FileTable>;
}

/// File table
pub struct FileTable {
    inner: Mutex<Vec<Option<Arc<FileHandle>>>>,
}

impl Default for FileTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FileTable {
    fn clone(&self) -> Self {
        let inner = self.inner.lock();
        FileTable {
            inner: Mutex::new(inner.iter().cloned().collect()),
        }
    }
}

impl FileTable {
    /// Create a file table
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }

    /// Inserts a file handle
    pub fn insert(&self, file_handle: Arc<FileHandle>) -> AxResult<usize> {
        let mut fd_list = self.inner.lock();
        if let Some(fd) = fd_list.iter_mut().enumerate().find_map(|(fd, handle)| {
            if handle.is_none() {
                *handle = Some(file_handle.clone());
                Some(fd)
            } else {
                None
            }
        }) {
            Ok(fd)
        } else {
            fd_list.push(Some(file_handle.clone()));
            Ok(fd_list.len() - 1)
        }
    }

    /// Finds by file descriptor
    pub fn find(&self, fd: usize) -> AxResult<Arc<FileHandle>> {
        let fd_list = self.inner.lock();
        if fd >= fd_list.len() {
            return ax_err!(BadFileDescriptor);
        }
        if let Some(handle) = &fd_list[fd] {
            Ok(handle.clone())
        } else {
            ax_err!(BadFileDescriptor)
        }
    }

    /// Remove by file descriptor
    pub fn remove(&self, fd: usize) -> AxResult<()> {
        let mut fd_list = self.inner.lock();
        *fd_list.get_mut(fd).ok_or(AxError::BadFileDescriptor)? = None;
        Ok(())
    }
}

/// Handles all I/O syscalls
pub fn syscall_handler(id: usize, params: [usize; 6]) -> isize {
    let ret = match id & SYS_CLASS {
        SYS_CLASS_FILE => {
            let fd = params[0];
            match id & SYS_ARG {
                SYS_ARG_SLICE => {
                    match id {
                        SYS_FMAP => ax_err!(Unsupported), // TODO
                        _ => file_op_slice(
                            id,
                            fd,
                            &copy_slice_from_user(params[1].into(), params[2]),
                        ),
                    }
                }
                SYS_ARG_MSLICE => {
                    match id {
                        SYS_FSTAT => file_op_slice_mut(id, fd, params[1], params[2]), // TODO: Support UNIX stat struct
                        _ => file_op_slice_mut(id, fd, params[1], params[2]),
                    }
                }
                _ => match id {
                    SYS_CLOSE => close(fd),
                    SYS_DUP => dup(
                        params[0],
                        &copy_slice_from_user(params[1].into(), params[2]),
                    ),
                    SYS_DUP2 => ax_err!(Unsupported),    // TODO
                    SYS_FCNTL => ax_err!(Unsupported),   // TODO
                    SYS_FRENAME => ax_err!(Unsupported), // TODO
                    SYS_FUNMAP => ax_err!(Unsupported),  // TODO
                    _ => file_op(id, fd, params[1], params[2]),
                },
            }
        }
        SYS_CLASS_PATH => match id {
            SYS_OPEN => open(
                &axmem::copy_str_from_user(params[0].into(), params[1]),
                params[2],
            ),
            SYS_RMDIR => ax_err!(Unsupported),  // TODO
            SYS_UNLINK => ax_err!(Unsupported), // TODO
            _ => ax_err!(Unsupported),
        },
        _ => ax_err!(Unsupported),
    };
    to_ret_code(ret)
}

fn file_op_slice(id: usize, fd: usize, slice: &[u8]) -> AxResult<usize> {
    file_op(id, fd, slice.as_ptr() as usize, slice.len())
}
fn file_op_slice_mut(id: usize, fd: usize, ptr: usize, len: usize) -> AxResult<usize> {
    let buffer: Vec<u8> = alloc::vec![0; len];
    let buffer_slice = buffer.as_slice();
    let ret = file_op(id, fd, buffer_slice.as_ptr() as usize, buffer_slice.len())?;
    copy_byte_buffer_to_user(0, ptr as *const u8, buffer_slice);
    Ok(ret)
}

/// Initializes scheme
pub fn init_scheme() {
    GLOBAL_SCHEME_LIST.init_by(Mutex::new(SchemeList::new_init()));
    open("stdin:", 0).unwrap();
    open("stdout:", 0).unwrap();
    open("stdout:", 0).unwrap();
}

fn insert_fd(fd: Arc<FileHandle>) -> AxResult<usize> {
    let file_table = call_interface!(CurrentFileTable::current_file_table);
    file_table.insert(fd)
}

fn find_fd(fd: usize) -> AxResult<Arc<FileHandle>> {
    let file_table = call_interface!(CurrentFileTable::current_file_table);
    file_table.find(fd)
}

// TODO: all flags
fn open(path: &str, options: usize) -> AxResult<usize> {
    let mut path_split = path.splitn(2, ':');
    let (scheme, path) = match (path_split.next(), path_split.next()) {
        (Some(scheme), Some(path)) => (scheme, path),
        (Some(path), None) => ("file", path),
        _ => return ax_err!(NotFound),
    };
    trace!("Open {}:{}", scheme, path);
    let scheme_id = schemes().find_name(scheme).ok_or(AxError::NotFound)?;
    let scheme = schemes().find_id(scheme_id).unwrap();

    let file_handle = Arc::new(FileHandle {
        scheme_id,
        file_id: scheme.open(path, options, 0, 0)?,
    });

    insert_fd(file_handle)
}

fn file_op(op: usize, fd: usize, c: usize, d: usize) -> AxResult<usize> {
    let handle = find_fd(fd)?;
    let scheme = schemes().find_id(handle.scheme_id).unwrap();
    let file = handle.file_id;
    let mut packet = Packet {
        a: op,
        b: file,
        c,
        d,
        id: 0,
        pid: current().id().as_u64() as usize,
        uid: 0,
        gid: 0,
    };
    scheme.handle(&mut packet);
    Ok(packet.a)
}

fn close(fd: usize) -> AxResult<usize> {
    let handle = find_fd(fd)?;

    let scheme = schemes().find_id(handle.scheme_id).unwrap();

    let ret = scheme.close(handle.file_id)?;

    {
        let file_table = call_interface!(CurrentFileTable::current_file_table);
        file_table.remove(fd)?;
    }

    Ok(ret)
}

fn dup_inner(fd: usize, buf: &[u8]) -> AxResult<Arc<FileHandle>> {
    let handle = find_fd(fd)?;

    if buf.is_empty() {
        Ok(handle)
    } else {
        let scheme = schemes().find_id(handle.scheme_id).unwrap();
        let new_id = scheme.dup(handle.file_id, buf)?;

        Ok(Arc::new(FileHandle {
            scheme_id: handle.scheme_id,
            file_id: new_id,
        }))
    }
}

fn dup(fd: usize, buf: &[u8]) -> AxResult<usize> {
    let handle = dup_inner(fd, buf)?;
    insert_fd(handle)
}

/// Id of a Scheme
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct SchemeId(usize);

/// Container of all scheme implementations
pub struct SchemeList {
    map: BTreeMap<SchemeId, Arc<dyn KernelScheme + Sync + Send>>,
    names: BTreeMap<Box<str>, SchemeId>,
    next_id: usize,
}

impl SchemeList {
    fn new() -> Self {
        SchemeList {
            map: BTreeMap::new(),
            names: BTreeMap::new(),
            next_id: 1,
        }
    }
    fn new_init() -> Self {
        let mut result = Self::new();
        // TODO: add basic Schemes
        result.insert("", Arc::new(RootScheme::new()));
        result.insert("stdout", Arc::new(Stdout::new()));
        result.insert("stdin", Arc::new(Stdin));
        result
    }
    /// Inserts a scheme
    pub fn insert(&mut self, name: &str, scheme: Arc<dyn KernelScheme + Sync + Send>) {
        let id = SchemeId(self.next_id);
        trace!("insert {} scheme", name);
        self.next_id += 1;
        assert!(self
            .names
            .insert(name.to_string().into_boxed_str(), id)
            .is_none());
        assert!(self.map.insert(id, scheme).is_none());
    }

    /// Finds a scheme by name
    pub fn find_name(&self, name: &str) -> Option<SchemeId> {
        self.names.get(name).copied()
    }
    /// Finds a scheme by id
    pub fn find_id(&self, id: SchemeId) -> Option<Arc<dyn KernelScheme + Sync + Send>> {
        self.map.get(&id).cloned()
    }
}

static GLOBAL_SCHEME_LIST: LazyInit<Mutex<SchemeList>> = LazyInit::new();

/// Gets all scheme list with lock
pub fn schemes() -> MutexGuard<'static, SchemeList> {
    GLOBAL_SCHEME_LIST.lock()
}

use scheme::Packet;
pub use scheme::Scheme;
use syscall_number::*;
/// Schemes implemented in kernel
pub trait KernelScheme: Scheme {}

pub mod dev;
mod io;
mod root;
mod user;
use io::{Stdin, Stdout};

use self::root::RootScheme;
pub use user::FindAddrSpace;
