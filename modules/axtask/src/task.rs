use alloc::{boxed::Box, sync::Arc};
use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use core::{alloc::Layout, cell::UnsafeCell, fmt, ptr::NonNull};

use axconfig::TASK_STACK_SIZE;
use axhal::arch::TaskContext;
use memory_addr::VirtAddr;

use crate::{AxTask, AxTaskRef};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(u64);

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TaskState {
    Runnable = 1,
    Blocked = 2,
    Exited = 3,
}

pub struct TaskInner {
    id: TaskId,
    name: &'static str,

    entry: Option<*mut dyn FnOnce()>,
    state: AtomicU8,

    kstack: Option<TaskStack>,
    ctx: UnsafeCell<TaskContext>,
}

impl TaskId {
    const IDLE_TASK_ID: Self = Self(0);

    fn new() -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl const From<u8> for TaskState {
    fn from(state: u8) -> Self {
        match state {
            1 => Self::Runnable,
            2 => Self::Blocked,
            3 => Self::Exited,
            _ => unreachable!(),
        }
    }
}

unsafe impl Send for TaskInner {}
unsafe impl Sync for TaskInner {}

impl TaskInner {
    pub const fn id(&self) -> TaskId {
        self.id
    }

    pub const fn name(&self) -> &str {
        self.name
    }

    pub fn id_name(&self) -> alloc::string::String {
        alloc::format!("Task({}, {:?})", self.id.as_u64(), self.name)
    }
}

// private methods
impl TaskInner {
    fn new_common(id: TaskId, name: &'static str) -> Self {
        Self {
            id,
            name,
            entry: None,
            state: AtomicU8::new(TaskState::Runnable as u8),
            kstack: None,
            ctx: UnsafeCell::new(TaskContext::new()),
        }
    }

    pub(crate) fn new<F>(entry: F, name: &'static str) -> AxTaskRef
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common(TaskId::new(), name);
        debug!("new task: {}", t.id_name());
        let kstack = TaskStack::alloc(TASK_STACK_SIZE);
        t.entry = Some(Box::into_raw(Box::new(entry)));
        t.ctx.get_mut().init(task_entry as usize, kstack.top());
        t.kstack = Some(kstack);
        Arc::new(AxTask::new(t))
    }

    pub(crate) fn new_init() -> AxTaskRef {
        // init_task does not change PC and SP, so `entry` and `stack` fields are not used.
        Arc::new(AxTask::new(Self::new_common(TaskId::new(), "init")))
    }

    pub(crate) fn new_idle() -> AxTaskRef {
        let mut t = Self::new_common(TaskId::IDLE_TASK_ID, "idle");
        let kstack = TaskStack::alloc(TASK_STACK_SIZE);
        t.ctx.get_mut().init(idle_entry as usize, kstack.top());
        t.kstack = Some(kstack);
        Arc::new(AxTask::new(t))
    }

    pub(crate) fn state(&self) -> TaskState {
        self.state.load(Ordering::SeqCst).into()
    }

    pub(crate) fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::SeqCst)
    }

    pub(crate) fn is_runnable(&self) -> bool {
        matches!(self.state(), TaskState::Runnable)
    }

    pub(crate) const fn is_idle(&self) -> bool {
        self.id.as_u64() == TaskId::IDLE_TASK_ID.as_u64()
    }

    pub(crate) const fn ctx_mut_ptr(&self) -> *mut TaskContext {
        self.ctx.get()
    }
}

impl fmt::Debug for TaskInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TaskInner")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state())
            .finish()
    }
}

impl Drop for TaskInner {
    fn drop(&mut self) {
        debug!("task drop: {}", self.id_name());
    }
}

struct TaskStack {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl TaskStack {
    pub fn alloc(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 16).unwrap();
        Self {
            ptr: NonNull::new(unsafe { alloc::alloc::alloc(layout) }).unwrap(),
            layout,
        }
    }

    pub const fn top(&self) -> VirtAddr {
        unsafe { core::mem::transmute(self.ptr.as_ptr().add(self.layout.size())) }
    }
}

impl Drop for TaskStack {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

extern "C" fn idle_entry() -> ! {
    unsafe { crate::RUN_QUEUE.force_unlock() };
    loop {
        crate::yield_now();
        debug!("idle task: waiting for IRQs...");
        axhal::arch::wait_for_irqs();
    }
}

extern "C" fn task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { crate::RUN_QUEUE.force_unlock() };
    // TODO: enable IRQ
    let task = crate::current();
    if let Some(entry) = task.entry {
        unsafe { Box::from_raw(entry)() };
    }
    crate::exit(0);
}
