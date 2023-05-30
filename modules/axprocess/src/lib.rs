#![no_std]

extern crate alloc;
#[macro_use]
extern crate axlog;

use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{sync::Arc, vec, vec::Vec};
use axmem::AddrSpace;
use axscheme::FileTable;
use axtask::{current, AxTaskRef};
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Pid(u64);

impl Pid {
    fn alloc() -> Pid {
        static ID: AtomicU64 = AtomicU64::new(1);
        Pid(ID.fetch_add(1, Ordering::SeqCst))
    }
}

struct AxProcess {
    pid: Pid,
    addr_space: Arc<AddrSpace>,
    file_table: Arc<FileTable>,
    tasks: Arc<SpinNoIrq<Vec<AxTaskRef>>>,
}

static PROCESS_TABLE: LazyInit<SpinNoIrq<Vec<AxProcess>>> = LazyInit::new();

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

    let process_table = vec![AxProcess {
        pid: Pid::alloc(),
        addr_space: Arc::new(user_space),
        file_table: Arc::new(FileTable::new()),
        tasks: Arc::new(SpinNoIrq::new(vec![])),
    }];

    PROCESS_TABLE.init_by(SpinNoIrq::new(process_table));
}

struct CurrentAddrSpaceImpl();
struct CurrentFileTableImpl();

#[crate_interface::impl_interface]
impl axmem::CurrentAddrSpace for CurrentAddrSpaceImpl {
    fn current_addr_space() -> Arc<AddrSpace> {
        PROCESS_TABLE.lock()[0].addr_space.clone()
    }
}

#[crate_interface::impl_interface]
impl axscheme::CurrentFileTable for CurrentFileTableImpl {
    fn current_file_table() -> Arc<FileTable> {
        PROCESS_TABLE.lock()[0].file_table.clone()
    }
}
