//! Process implementation
#![no_std]
#![feature(drain_filter)]

extern crate alloc;
#[macro_use]
extern crate axlog;

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

use alloc::{
    sync::{Arc, Weak},
    vec,
    vec::Vec,
};
use axerrno::AxResult;
use axmem::AddrSpace;
use axscheme::FileTable;
use axtask::{current, current_task, yield_now, AxTaskRef};
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
    parent: SpinNoIrq<Weak<AxProcess>>,
    child: SpinNoIrq<Vec<Arc<AxProcess>>>,
    addr_space: Arc<AddrSpace>,
    file_table: Arc<FileTable>,
    tasks: SpinNoIrq<Vec<AxTaskRef>>,
    exit_code: AtomicI32,
    exited: AtomicBool,
}

static PROCESS_TABLE: LazyInit<SpinNoIrq<Vec<Arc<AxProcess>>>> = LazyInit::new();
static INIT_PROCESS: LazyInit<Arc<AxProcess>> = LazyInit::new();

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

    let init_process = Arc::new(AxProcess {
        pid: Pid::alloc(),
        parent: SpinNoIrq::new(Weak::new()),
        child: SpinNoIrq::new(Vec::new()),
        addr_space: Arc::new(user_space),
        file_table: Arc::new(FileTable::new()),
        tasks: SpinNoIrq::new(Vec::new()),
        exit_code: AtomicI32::new(0),
        exited: AtomicBool::new(false),
    });

    let process_table = vec![init_process.clone()];

    PROCESS_TABLE.init_by(SpinNoIrq::new(process_table));
    INIT_PROCESS.init_by(init_process);
}
/// Initializes task structure after axtask is inited
pub fn post_task_init() {
    PROCESS_TABLE.lock()[0].tasks.lock().push(current_task());
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
    trace!("fork");
    let current = current_process();
    let res = Arc::new(AxProcess {
        pid: Pid::alloc(),
        parent: SpinNoIrq::new(Arc::downgrade(&current)),
        child: SpinNoIrq::new(Vec::new()),
        addr_space: Arc::new(current.addr_space.as_ref().clone()),
        file_table: Arc::new(current.file_table.as_ref().clone()),
        tasks: SpinNoIrq::new(Vec::new()),
        exit_code: AtomicI32::new(0),
        exited: AtomicBool::new(false),
    });

    current.child.lock().push(res.clone());
    let task = axtask::handle_fork(res.pid.0, res.addr_space.clone());
    res.tasks.lock().push(task);
    PROCESS_TABLE.lock().push(res.clone());
    res.pid.0 as usize
}

/// push the task into process sturcture after `spawn` syscall
pub fn add_task(task: AxTaskRef) {
    current_process().tasks.lock().push(task)
}

/// Exit current process
pub fn exit_current(code: i32) {
    trace!("exit");
    let task = current();
    let process = current_process();
    let id = process
        .tasks
        .lock()
        .iter()
        .enumerate()
        .find(|(_, task_i)| task.id() == task_i.id())
        .unwrap()
        .0;

    if id == 0 {
        if process.pid.0 == 1 {
            // Directly call termination in axtask
            return;
        }
        // main process
        // Wait all tasks to complete
        // TODO: use signals to kill
        process
            .tasks
            .lock()
            .iter()
            .skip(1) // exclude current task
            .for_each(|task_i| {
                task_i.join();
            });

        // make all child zombie
        process.child.lock().iter_mut().for_each(|child_process| {
            *child_process.parent.lock() = Arc::downgrade(&INIT_PROCESS);
            INIT_PROCESS.child.lock().push(child_process.clone())
        });
        process.child.lock().clear();
        process.tasks.lock().clear();

        process.exit_code.store(code, Ordering::Release);
        process.exited.store(true, Ordering::Release);
        // remove from process table
        PROCESS_TABLE
            .lock()
            .drain_filter(|process_inner| process_inner.pid == process.pid);

        // There is no need to prevent resource (memory space) to be released before task switch
        // as one reference is held by its parent
    } else {
        // others
        let task = process.tasks.lock().remove(id);
        task.on_exit(|vaddr| {
            process.addr_space.lock().remove_region(vaddr).unwrap();
        });
        debug!("{}", process.tasks.lock().len());
    }
}

/// Wait for a child process to stop.
pub fn wait(_pid: u64) -> (u64, i32) {
    trace!("wait");
    let process = current_process();
    loop {
        if let Some((id, process_i)) = process
            .child
            .lock()
            .iter()
            .enumerate()
            .find(|(_, process_i)| process_i.exited.load(Ordering::Acquire))
        {
            let ret = (process_i.pid.0, process_i.exit_code.load(Ordering::Acquire));
            process.child.lock().remove(id);
            return ret;
        }

        yield_now();
    }
}

/// exec syscall
/// only returns on error
pub fn exec(elf_data: Vec<u8>) -> isize {
    trace!("exec");
    fn exec_inner(elf_data: Vec<u8>) -> AxResult<()> {
        let process = current_process();
        process.addr_space.init_exec(&elf_data)?;
        process.file_table.reset();
        process.tasks.lock().clear();
        Ok(())
        // elf_data should be successfully dropped
    }

    if exec_inner(elf_data).is_err() {
        return -1;
    }
    let process = current_process();

    axtask::handle_exec(|task| process.tasks.lock().push(task))
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
