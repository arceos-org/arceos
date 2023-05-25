extern crate alloc;
use alloc::{collections::BTreeMap, sync::Arc};
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::ToString;
use axmem::{copy_slice_from_user, copy_byte_buffer_to_user};
use axtask::current;
use lazy_init::LazyInit;
use axsync::{Mutex, MutexGuard};

pub struct FileHandle {
    pub scheme_id: SchemeId,
    pub file_id: usize,
}

static GLOBAL_FD_LIST: LazyInit<Mutex<Vec<Option<Arc<FileHandle>>>>> = LazyInit::new();

pub fn syscall_handler(id: usize, params: [usize; 6]) -> isize {
    match id & SYS_CLASS {
        SYS_CLASS_FILE => {
            let fd = params[0];
            match id & SYS_ARG {
                SYS_ARG_SLICE => {
                    match id {
                        SYS_FMAP => -1, // TODO
                        _ => {
                            file_op_slice(id, fd, &copy_slice_from_user(params[1].into(), params[2]))
                        }
                    }
                }
                SYS_ARG_MSLICE => {
                    match id {
                        SYS_FSTAT => -1, // TODO
                        _ => {
                            file_op_slice_mut(id, fd, params[1], params[2])
                        }
                    }
                }
                _ => match id {
                        SYS_CLOSE => close(fd),
                        SYS_DUP => -1, // TODO
                        SYS_DUP2 => -1, // TODO
                        SYS_FCNTL => -1, // TODO
                        SYS_FRENAME => -1, // TODO
                        SYS_FUNMAP => -1, // TODO
                        _ => file_op(id, fd, params[1], params[2]),
                }

            }
            
        },
        SYS_CLASS_PATH => match id {
            SYS_OPEN => open(&axmem::copy_str_from_user(params[0].into(), params[1]), params[2]) as isize,
            SYS_RMDIR => -1, // TODO
            SYS_UNLINK => -1, // TODO
            _ => -1,
        },
        _ => -1,
    }
}

fn file_op_slice(id: usize, fd: usize, slice: &[u8]) -> isize {
    file_op(id, fd, slice.as_ptr() as usize, slice.len())
}
fn file_op_slice_mut(id: usize, fd: usize, ptr: usize, len: usize) -> isize {
    let buffer: Vec<u8> = alloc::vec![0; len];
    let buffer_slice = buffer.as_slice();
    let ret = file_op(id, fd, buffer_slice.as_ptr() as usize, buffer_slice.len());
    copy_byte_buffer_to_user(0, ptr as *const u8, buffer_slice);
    ret
}

pub fn init_scheme() {
    GLOBAL_SCHEME_LIST.init_by(Mutex::new(SchemeList::new_init()));
    GLOBAL_FD_LIST.init_by(Mutex::new(Vec::new()));    
    open("stdin:", 0);
    open("stdout:", 0);
    open("stdout:", 0);   
}
// TODO: all flags
pub fn open(path: &str, options: usize) -> usize {
    let mut path_split = path.splitn(2, ":");
    let (scheme, path) = match (path_split.next(), path_split.next()) {
        (Some(scheme), Some(path)) => (scheme, path),
        (Some(path), None) => ("file", path),
        _ => panic!("Invalid URL path!"),
    };
    trace!("Open {}:{}", scheme, path);
    let scheme_id = schemes().find_name(scheme).expect("Scheme not found");
    let scheme = schemes().find_id(scheme_id).unwrap();

    let file_handle = Arc::new(FileHandle {
        scheme_id,
        file_id: scheme.open(path, options, 0, 0).unwrap()
    });
    
    let mut fd_list = GLOBAL_FD_LIST.lock();
    if let Some(fd) = fd_list.iter_mut().enumerate().find_map(|(fd, handle)| {
        if handle.is_none() {
            *handle = Some(file_handle.clone());
            Some(fd)
        } else {
            None
        }
    }) {
        fd
    } else {
        fd_list.push(Some(file_handle.clone()));
        fd_list.len() - 1
    }
}

pub fn find_fd(fd: usize) -> Arc<FileHandle> {
    let fd_list = GLOBAL_FD_LIST.lock();
    if fd >= fd_list.len() {
        panic!("Invalid FD!");
    }
    if let Some(handle) = &fd_list[fd] {
        handle.clone()
    } else {
        panic!("Invalid FD!");
    }
}

pub fn file_op(op: usize, fd: usize, c: usize, d: usize) -> isize {
    let handle = find_fd(fd);
    let scheme = schemes().find_id(handle.scheme_id).unwrap();
    let file = handle.file_id;
    let mut packet = Packet {
        a: op,
        b: file,
        c, d,
        id: 0,
        pid: current().id().as_u64() as usize,
        uid: 0,
        gid: 0,
    };
    scheme.handle(&mut packet);
    packet.a as isize       
}

pub fn close(fd: usize) -> isize {
    let handle = find_fd(fd);

    let scheme = schemes().find_id(handle.scheme_id).unwrap();
    
    let ret = scheme.close(handle.file_id).map(|x| x as isize).unwrap_or_else(|e| -(e as i32 as isize));
    {
        let mut fd_list = GLOBAL_FD_LIST.lock();
        fd_list[fd] = None;
    }
    ret        
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct SchemeId(usize);

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
    pub fn insert(&mut self, name: &str, scheme: Arc<dyn KernelScheme + Sync + Send>) {
        let id = SchemeId(self.next_id);
        trace!("insert {} scheme", name);
        self.next_id += 1;        
        assert!(self.names.insert(name.to_string().into_boxed_str(), id).is_none());        
        assert!(self.map.insert(id, scheme).is_none());
    }

    pub fn find_name(&self, name: &str) -> Option<SchemeId>{
        self.names.get(name).copied()
    }
    pub fn find_id(&self, id: SchemeId) -> Option<Arc<dyn KernelScheme + Sync + Send>> {
        self.map.get(&id).cloned()
    }
}

static GLOBAL_SCHEME_LIST: LazyInit<Mutex<SchemeList>> = LazyInit::new();

pub fn schemes() -> MutexGuard<'static, SchemeList> {
    GLOBAL_SCHEME_LIST.lock()
}


use scheme::Packet;
pub use scheme::Scheme;
use syscall_number::*;
pub trait KernelScheme: Scheme {
}

mod root;
mod user;
mod io;
pub mod dev;
use io::{Stdin, Stdout};

use self::root::RootScheme;
