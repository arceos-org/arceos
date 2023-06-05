//! Process implementation
#![no_std]

extern crate alloc;
#[macro_use]
extern crate axlog;

use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use axmem::AddrSpace;
use axscheme::FileTable;
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Pid(u64);

impl From<u64> for Pid {
    fn from(value: u64) -> Self {
        Pid(value)
    }
}
impl Pid {
    fn alloc() -> Pid {
        static ID: AtomicU64 = AtomicU64::new(1);
        Pid(ID.fetch_add(1, Ordering::SeqCst))
    }
}

struct AxProcess {
    pid: Pid,
    parent: Weak<AxProcess>,
    child: SpinNoIrq<Vec<Arc<AxProcess>>>,
    addr_space: Arc<AddrSpace>,
    file_table: Arc<FileTable>,
}

static PROCESS_TABLE: LazyInit<SpinNoIrq<Vec<Arc<AxProcess>>>> = LazyInit::new();

/// Initializes process structures
pub fn init() {
    extern "C" {
        fn ustart();
        fn uend();
    }

    let user_elf: &[u8] = unsafe {
        let len = (uend as usize) - (ustart as usize);
        core::slice::from_raw_parts(ustart as *const _, len)
    };

    debug!("{:x} {:x}", ustart as usize, user_elf.len());

    let user_space = AddrSpace::init_global(user_elf).unwrap();

    let process_table = vec![Arc::new(AxProcess {
        pid: Pid::alloc(),
        parent: Weak::new(),
        child: SpinNoIrq::new(Vec::new()),
        addr_space: Arc::new(user_space),
        file_table: Arc::new(FileTable::new()),
    })];

    PROCESS_TABLE.init_by(SpinNoIrq::new(process_table));
}

fn find(pid: Pid) -> Option<Arc<AxProcess>> {
    PROCESS_TABLE
        .lock()
        .iter()
        .find(|process| process.pid == pid)
        .cloned()
}
fn current_process() -> Arc<AxProcess> {
    match axtask::current_pid() {
        Some(pid) => find(pid.into()).unwrap(),
        None => PROCESS_TABLE.lock()[0].clone(),
    }
}

/// Forks the current process
pub fn fork() -> usize {
    let current = current_process();
    let res = Arc::new(AxProcess {
        pid: Pid::alloc(),
        parent: Arc::downgrade(&current),
        child: SpinNoIrq::new(Vec::new()),
        addr_space: Arc::new(current.addr_space.as_ref().clone()),
        file_table: Arc::new(current.file_table.as_ref().clone()),
    });

    current.child.lock().push(res.clone());
    axtask::handle_fork(res.pid.0, res.addr_space.clone());
    PROCESS_TABLE.lock().push(res.clone());
    res.pid.0 as usize
}

struct CurrentAddrSpaceImpl;
struct CurrentFileTableImpl;
struct FindAddrSpaceImpl;

#[crate_interface::impl_interface]
impl axmem::CurrentAddrSpace for CurrentAddrSpaceImpl {
    fn current_addr_space() -> Arc<AddrSpace> {
        current_process().addr_space.clone()
    }
}

#[crate_interface::impl_interface]
impl axscheme::CurrentFileTable for CurrentFileTableImpl {
    fn current_file_table() -> Arc<FileTable> {
        current_process().file_table.clone()
    }
}

#[crate_interface::impl_interface]
impl axscheme::FindAddrSpace for FindAddrSpaceImpl {
    fn find_addr_space(pid: u64) -> Option<Arc<AddrSpace>> {
        find(pid.into()).map(|x| x.addr_space.clone())
    }
}
