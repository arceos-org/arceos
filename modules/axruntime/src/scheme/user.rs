extern crate alloc;
use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{sync::Weak, collections::{VecDeque, BTreeMap}};
use axerrno::{AxResult, ax_err};
use axsync::Mutex;
use axtask::current;
use scheme::{Scheme, Packet};
use alloc::boxed::Box;

use super::KernelScheme;


pub struct UserInner {
    id: usize,
    name: Box<str>,
    next_id: AtomicU64,
    requests: Mutex<VecDeque<Packet>>,
    /// response from server, Key is id, Value is return value.
    response: Mutex<BTreeMap<u64, usize>>,
}

impl UserInner {
    pub fn new(id: usize, path: Box<str>) -> UserInner {
        UserInner {
            id,
            name: path,
            next_id: 1.into(),
            requests: Mutex::new(VecDeque::new()),
            response: Mutex::new(BTreeMap::new()),
        }
    }
    /// read a request from the clients to the server
    pub fn scheme_read(&self, buf: &mut [u8]) -> AxResult<usize> {
        let buf: &mut [Packet] = unsafe {
            let ptr = buf.as_mut_ptr() as *mut Packet;
            core::slice::from_raw_parts_mut(ptr, buf.len() / core::mem::size_of::<Packet>())
        };
        for copy_item in buf.iter_mut() {
            if let Some(request) = self.requests.lock().pop_front() {
                core::mem::replace(copy_item, request);
            } else {
                // TODO: option to return EAGAIN
                // TODO: use blocking instead of yield
                axtask::yield_now();
            }
        }
        Ok(buf.len() * core::mem::size_of::<Packet>())
    }
    /// Write a response form the server
    pub fn scheme_write(&self, buf: &[u8]) -> AxResult<usize> {
        if buf.len() % core::mem::size_of::<Packet>() != 0 {
            return ax_err!(InvalidData);
        }
        let buf: &[Packet] = unsafe {
            let ptr = buf.as_ptr() as *const Packet;
            core::slice::from_raw_parts(ptr, buf.len() / core::mem::size_of::<Packet>())
        };
        for result_item in buf.iter() {
            self.response.lock().insert(result_item.id, result_item.a);
        }       
        Ok(buf.len() * core::mem::size_of::<Packet>())
    }

    pub fn handle_request(&self, a: usize, b: usize, c: usize, d: usize) -> AxResult<usize> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let packet = Packet {
            id,
            uid: 0,
            gid: 0,
            pid: current().id().as_u64() as usize,
            a, b, c, d
        };

        self.requests.lock().push_back(packet);
        loop {
            if let Some(value) = self.response.lock().remove(&id) {
                return Ok(value);
            } else {
                axtask::yield_now();
            }
        }
    }
}

pub struct UserScheme {
    inner: Weak<UserInner>,    
}
impl UserScheme {
    pub fn new(inner: Weak<UserInner>) -> Self {
        UserScheme {
            inner
        }
    }
}
impl Scheme for UserScheme {
    
}
impl KernelScheme for UserScheme {}
