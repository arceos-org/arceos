extern crate alloc;
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{sync::Arc, collections::BTreeMap};
use axerrno::{ax_err, AxError, AxResult};
use axsync::Mutex;
use scheme::Scheme;
use alloc::string::ToString;
use syscall_number::io::OpenFlags;

use super::{user::{UserInner, UserScheme}, schemes, KernelScheme};


pub struct RootScheme {
    handles: Mutex<BTreeMap<usize, RootHandle>>,
    next_id: AtomicUsize,
}
enum RootHandle {
    Scheme(Arc<UserInner>)
}
impl RootScheme {
    pub fn new() -> Self {
        RootScheme {
            handles: Mutex::new(BTreeMap::new()),
            next_id: 0.into(),
        }
    }
}
impl Scheme for RootScheme {
    fn open(&self, path: &str, flags: usize, _uid: u32, _gid: u32) -> AxResult<usize> {
        let path = path.trim_matches('/');
        let flags = OpenFlags::from_bits(flags).ok_or(AxError::InvalidInput)?;

        if flags.contains(OpenFlags::CREATE) {

            // Create a user scheme
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let inner = {
                let path_box = path.to_string().into_boxed_str();
                let inner = Arc::new(UserInner::new(id, path_box));
                schemes().insert(&path, Arc::new(UserScheme::new(Arc::downgrade(&inner))));
                inner
            };
            
            self.handles.lock().insert(id, RootHandle::Scheme(inner));
            trace!("Root Scheme: create {} -> {}", path, id);
            Ok(id)
        } else if path.is_empty() {
            // list all schemes
            todo!();
        } else {
            // in redox, this was implemented as a unreadable and unwritable file,
            // we simply reject it.
            ax_err!(InvalidInput)
        }
    }

    fn close(&self, id: usize) -> AxResult<usize> {
        let handle = self.handles.lock().remove(&id).ok_or(AxError::BadFileDescriptor)?;

        match handle {
            RootHandle::Scheme(_inner) => {
                // TODO: remove a scheme
            }
        }
        Ok(0)
    }

    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        let handles = self.handles.lock() ;
        let handle = handles.get(&id).ok_or(AxError::BadFileDescriptor)?;
        trace!("Root Scheme {}: read", id);
        match handle {
            RootHandle::Scheme(inner) => {
                // There is a blocking situation, so lock must be dropped.
                let inner = inner.clone();
                drop(handles);
                inner.scheme_read(buf)
            }
        }
    }

    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        let handles = self.handles.lock() ;
        let handle = handles.get(&id).ok_or(AxError::BadFileDescriptor)?;
        trace!("Root Scheme {}: write", id);
        match handle {
            RootHandle::Scheme(inner) => {
                inner.scheme_write(buf)
            }
        }
    }
}
impl KernelScheme for RootScheme {}
