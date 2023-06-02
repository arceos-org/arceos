extern crate alloc;
use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use alloc::{boxed::Box, sync::Arc};
use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Weak,
};
use axerrno::{ax_err, from_ret_code, AxError, AxResult};
use axhal::{mem::VirtAddr, paging::MappingFlags};
use axmem::{AddrSpace};
use axsync::Mutex;
use axtask::{current, current_pid};
use scheme::{Packet, Scheme};
use syscall_number::{SYS_CLOSE, SYS_DUP, SYS_OPEN, SYS_READ, SYS_WRITE};

use super::KernelScheme;

pub struct UserInner {
    #[allow(unused)]
    id: usize,
    #[allow(unused)]
    name: Box<str>,
    next_id: AtomicU64,
    requests: Mutex<VecDeque<Packet>>,
    /// response from server, Key is id, Value is return value.
    response: Mutex<BTreeMap<u64, usize>>,
    #[cfg(feature = "process")]
    /// pid of the server
    pid: u64,
}

impl UserInner {
    pub fn new(id: usize, path: Box<str>) -> UserInner {
        UserInner {
            id,
            name: path,
            next_id: 1.into(),
            requests: Mutex::new(VecDeque::new()),
            response: Mutex::new(BTreeMap::new()),
            #[cfg(feature = "process")]
            pid: current_pid().unwrap(),
        }
    }
    /// read a request from the clients to the server
    pub fn scheme_read(&self, buf: &mut [u8]) -> AxResult<usize> {
        let buf: &mut [Packet] = unsafe {
            let ptr = buf.as_mut_ptr() as *mut Packet;
            core::slice::from_raw_parts_mut(ptr, buf.len() / core::mem::size_of::<Packet>())
        };
        for copy_item in buf.iter_mut() {
            loop {
                let request = self.requests.lock().pop_front();
                if let Some(request) = request {
                    trace!("Root recv {:#?}", request);
                    let _ = core::mem::replace(copy_item, request);
                    break;
                } else {
                    // TODO: option to return EAGAIN
                    // TODO: use blocking instead of yield
                    assert!(!self.requests.is_locked());
                    axtask::sleep(Duration::from_millis(1));
                }
            }
        }
        Ok(core::mem::size_of_val(buf))
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
            trace!("Root send {} -> {}", result_item.id, result_item.a);
            self.response.lock().insert(result_item.id, result_item.a);
        }
        Ok(core::mem::size_of_val(buf))
    }

    pub fn handle_request(&self, a: usize, b: usize, c: usize, d: usize) -> AxResult<usize> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let packet = Packet {
            id,
            uid: 0,
            gid: 0,
            pid: current().id().as_u64() as usize,
            a,
            b,
            c,
            d,
        };
        trace!("User Request: {:#?}", packet);
        self.requests.lock().push_back(packet);
        loop {
            let value = self.response.lock().remove(&id);
            if let Some(value) = value {
                return from_ret_code(value as isize);
            } else {
                assert!(!self.response.is_locked());
                axtask::sleep(Duration::from_millis(1));
            }
        }
    }
}

pub struct UserScheme {
    inner: Weak<UserInner>,
}
impl UserScheme {
    pub fn new(inner: Weak<UserInner>) -> Self {
        UserScheme { inner }
    }
}
impl Scheme for UserScheme {
    fn open(&self, path: &str, flags: usize, _uid: u32, _gid: u32) -> AxResult<usize> {
        let inner = self.inner.upgrade().ok_or(AxError::NotFound)?;
        let addr = ShadowMemory::new(path.as_bytes(), inner.pid)?;
        inner.handle_request(SYS_OPEN, addr.addr().into(), path.len(), flags)
    }
    fn read(&self, id: usize, buf: &mut [u8]) -> AxResult<usize> {
        let inner = self.inner.upgrade().ok_or(AxError::NotFound)?;
        let addr = ShadowMemoryMut::new(buf, inner.pid)?;
        inner.handle_request(SYS_READ, id, addr.addr().into(), addr.len())
    }
    fn write(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        let inner = self.inner.upgrade().ok_or(AxError::NotFound)?;
        let addr = ShadowMemory::new(buf, inner.pid)?;
        inner.handle_request(SYS_WRITE, id, addr.addr().into(), buf.len())
    }
    fn close(&self, id: usize) -> AxResult<usize> {
        let inner = self.inner.upgrade().ok_or(AxError::NotFound)?;
        inner.handle_request(SYS_CLOSE, id, 0, 0)
    }
    fn dup(&self, id: usize, buf: &[u8]) -> AxResult<usize> {
        let inner = self.inner.upgrade().ok_or(AxError::NotFound)?;
        let addr = ShadowMemory::new(buf, inner.pid)?;
        inner.handle_request(SYS_DUP, id, addr.addr().into(), buf.len())
    }
}
impl KernelScheme for UserScheme {}
struct TempMemory {
    page_start: VirtAddr,
    page_end: VirtAddr,
    pid: u64,
}
struct ShadowMemory {
    mem: TempMemory,
}
struct ShadowMemoryMut<'a> {
    mem: TempMemory,
    write_back: &'a mut [u8],
}
impl ShadowMemory {
    fn new(data: &[u8], pid: u64) -> AxResult<Self> {
        let page_start =
            mmap(pid, None, data.len(), MappingFlags::READ | MappingFlags::USER)?;
        let page_end = page_start + data.len();
        copy_buffer_to_user(pid, page_start, data);
        Ok(Self {
            mem: TempMemory {
                page_start,
                page_end,
                pid,
            },
        })
    }
    fn addr(&self) -> VirtAddr {
        self.mem.page_start
    }
}
impl<'a> ShadowMemoryMut<'a> {
    fn new(data: &'a mut [u8], pid: u64) -> AxResult<Self> {
        let page_start = mmap(
            pid, 
            None,
            data.len(),
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        )?;
        let page_end = page_start + data.len();
        Ok(Self {
            mem: TempMemory {
                page_start,
                page_end,
                pid
            },
            write_back: data,
        })
    }
    fn addr(&self) -> VirtAddr {
        self.mem.page_start
    }
    fn len(&self) -> usize {
        self.write_back.len()
    }
}
impl Drop for TempMemory {
    fn drop(&mut self) {
        munmap(
            self.pid,
            self.page_start,
            (self.page_end - self.page_start.into()).into(),
        )
        .unwrap();
    }
}
impl<'a> Drop for ShadowMemoryMut<'a> {
    fn drop(&mut self) {
        // TODO: optimize one copy time
        copy_buffer_from_user(self.mem.pid, self.mem.page_start, self.write_back);

    }
}

#[crate_interface::def_interface]
pub trait FindAddrSpace {
    fn find_addr_space(pid: u64) -> Option<Arc<AddrSpace>>;
}

fn mmap(pid: u64, addr: Option<VirtAddr>, len: usize, flags: MappingFlags) -> AxResult<VirtAddr> {
    let addr_space = call_interface!(FindAddrSpace::find_addr_space, pid).unwrap();
    let ret = addr_space.lock().mmap_page(addr, len, flags);
    ret
}

fn munmap(pid: u64, addr: VirtAddr, len: usize) -> AxResult<()> {
    let addr_space = call_interface!(FindAddrSpace::find_addr_space, pid).unwrap();
    let ret = addr_space.lock().munmap_page(addr, len);
    ret
}

fn copy_buffer_to_user(pid: u64, dest: VirtAddr, data: &[u8]) {
    let addr_space = call_interface!(FindAddrSpace::find_addr_space, pid).unwrap();
    let paddrs = addr_space.lock().translate_buffer(dest, data.len(), true).unwrap();
    let mut tot = 0;
    for paddr in paddrs {
        let len = paddr.len().min(data.len() - tot);
        if len > 0 {
            paddr[..len].copy_from_slice(&data[tot..tot+len]);
            tot += len;
        }
    }
}
fn copy_buffer_from_user(pid: u64, dest: VirtAddr, data: &mut [u8]) {
    let addr_space = call_interface!(FindAddrSpace::find_addr_space, pid).unwrap();
    let paddrs = addr_space.lock().translate_buffer(dest, data.len(), true).unwrap();
    let mut tot = 0;
    for paddr in paddrs {
        let len = paddr.len().min(data.len() - tot);
        if len > 0 {
            data[tot..tot+len].copy_from_slice(&paddr[..len]);
            tot += len;
        }
    }
}
