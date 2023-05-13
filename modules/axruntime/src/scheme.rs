extern crate alloc;
use alloc::{collections::BTreeMap, sync::Arc};
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::ToString;
use lazy_init::LazyInit;
use axsync::{Mutex, MutexGuard};

pub struct FileHandle {
    pub scheme_id: SchemeId,
}

static GLOBAL_FD_LIST: LazyInit<Mutex<Vec<Option<Arc<FileHandle>>>>> = LazyInit::new();

pub fn init_scheme() {
    GLOBAL_FD_LIST.init_by(Mutex::new(Vec::new()));
    GLOBAL_SCHEME_LIST.init_by(Mutex::new(SchemeList::new_init()));
}
// TODO: all flags
pub fn open(path: &str, _options: usize) -> usize {
    let mut path_split = path.splitn(2, ":");
    let (scheme, _path) = match (path_split.next(), path_split.next()) {
        (Some(scheme), Some(path)) => (scheme, path),
        (Some(path), None) => ("file", path),
        _ => panic!("Invalid URL path!"),
    };

    let scheme_id = schemes().find_name(scheme).expect("Scheme not found");
    let scheme = schemes().find_id(scheme_id).unwrap();

    let file_handle = Arc::new(FileHandle {
        scheme_id
    });
    scheme.handle();
    
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

pub fn file_op(fd: usize, _operation: usize) {
    let fd_list = GLOBAL_FD_LIST.lock();
    if fd >= fd_list.len() {
        panic!("Invalid FD!");
    }
    if let Some(handle) = &fd_list[fd] {
        let scheme = schemes().find_id(handle.scheme_id).unwrap();
        scheme.handle();
    } else {
        panic!("Invalid FD!");
    }
}

pub fn close(fd: usize) {
    let mut fd_list = GLOBAL_FD_LIST.lock();
    if fd >= fd_list.len() {
        panic!("Invalid FD!");
    }
    if let Some(handle) = &fd_list[fd] {
        let scheme = schemes().find_id(handle.scheme_id).unwrap();
        scheme.handle();        
    } else {
        panic!("Invalid FD!");
    }
    fd_list[fd] = None;
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
        let result = Self::new();
        // TODO: add basic Schemes
        result
    }
    pub fn insert(&mut self, name: &str, scheme: Arc<dyn KernelScheme + Sync + Send>) {
        let id = SchemeId(self.next_id);
        self.next_id += 1;
        self.names.insert(name.to_string().into_boxed_str(), id).unwrap();        
        self.map.insert(id, scheme).unwrap();
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


// TODO: now it's just a place holder
pub trait Scheme {
    fn handle(&self);
}
pub trait KernelScheme: Scheme {
}
