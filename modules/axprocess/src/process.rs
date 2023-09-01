//! 规定进程控制块内容
extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{collections::BTreeMap, string::String};
use axerrno::{AxError, AxResult};
use axhal::arch::TrapFrame;
use axhal::KERNEL_PROCESS_ID;
use axsync::Mutex;
use axtask::{AxTaskRef, TaskId, TaskInner};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
pub static TID2TASK: Mutex<BTreeMap<u64, AxTaskRef>> = Mutex::new(BTreeMap::new());
pub static PID2PC: Mutex<BTreeMap<u64, Arc<Process>>> = Mutex::new(BTreeMap::new());
const KERNEL_STACK_SIZE: usize = 0x40000;
pub struct Process {
    /// 进程号
    pid: u64,

    /// 父进程号
    pub parent: AtomicU64,

    /// 子进程
    pub children: Mutex<Vec<Arc<Process>>>,

    /// 所管理的线程
    pub tasks: Mutex<Vec<AxTaskRef>>,

    /// 地址空间

    /// 进程状态
    pub is_zombie: AtomicBool,

    /// 退出状态码
    pub exit_code: AtomicI32,
}

impl Process {
    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn get_parent(&self) -> u64 {
        self.parent.load(Ordering::Acquire)
    }

    pub fn set_parent(&self, parent: u64) {
        self.parent.store(parent, Ordering::Release)
    }

    pub fn get_exit_code(&self) -> i32 {
        self.exit_code.load(Ordering::Acquire)
    }

    pub fn set_exit_code(&self, exit_code: i32) {
        self.exit_code.store(exit_code, Ordering::Release)
    }

    pub fn get_zombie(&self) -> bool {
        self.is_zombie.load(Ordering::Acquire)
    }

    pub fn set_zombie(&self, status: bool) {
        self.is_zombie.store(status, Ordering::Release)
    }
}

impl Process {
    pub fn new(pid: u64, parent: u64) -> Self {
        Self {
            pid,
            parent: AtomicU64::new(parent),
            children: Mutex::new(Vec::new()),
            tasks: Mutex::new(Vec::new()),
            is_zombie: AtomicBool::new(false),
            exit_code: AtomicI32::new(0),
        }
    }
    /// 根据给定参数创建一个新的进程，作为应用程序初始进程
    pub fn init(args: Vec<String>) -> AxResult<AxTaskRef> {
        let path = args[0].clone();
        let new_process = Arc::new(Self::new(TaskId::new().as_u64(), KERNEL_PROCESS_ID));
        let new_task = TaskInner::new(|| {}, path, KERNEL_STACK_SIZE, new_process.pid());
        TID2TASK
            .lock()
            .insert(new_task.id().as_u64(), Arc::clone(&new_task));
        new_task.set_leader(true);

        let new_trap_frame = TrapFrame::app_init_context(0, 0);

        new_task.set_trap_context(new_trap_frame);

        new_process.tasks.lock().push(Arc::clone(&new_task));
        PID2PC
            .lock()
            .insert(new_process.pid(), Arc::clone(&new_process));
        // 将其作为内核进程的子进程
        match PID2PC.lock().get(&KERNEL_PROCESS_ID) {
            Some(kernel_process) => {
                kernel_process.children.lock().push(new_process);
            }
            None => {
                return Err(AxError::NotFound);
            }
        }
        Ok(new_task)
    }
}

impl Process {
    
}