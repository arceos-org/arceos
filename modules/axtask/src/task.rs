use alloc::{boxed::Box, sync::Arc};
use core::ops::Deref;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicU8, Ordering};
use core::{alloc::Layout, cell::UnsafeCell, fmt, ptr::NonNull};

#[cfg(feature = "preempt")]
use core::sync::atomic::AtomicUsize;

use crate::copy::__copy;
use axhal::arch::{TaskContext, TrapFrame};
use memory_addr::{align_up_4k, VirtAddr};
use riscv::asm;
const KERNEL_PROCESS_ID: u64 = 1;
use crate::time::TimeStat;
use crate::{AxTask, AxTaskRef};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(u64);

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TaskState {
    Running = 1,
    Ready = 2,
    Blocked = 3,
    Exited = 4,
}

pub struct TaskInner {
    id: TaskId,
    name: &'static str,
    is_idle: bool,
    is_init: bool,
    /// 所属进程
    process_id: u64,
    /// 是否是所属进程下的主线程
    is_leader: AtomicBool,
    /// 所包含的页表的token，内核的token统一为0
    page_table_token: usize,
    exit_code: AtomicI32,
    entry: Option<*mut dyn FnOnce()>,
    state: AtomicU8,

    in_wait_queue: AtomicBool,
    in_timer_list: AtomicBool,

    #[cfg(feature = "preempt")]
    need_resched: AtomicBool,
    #[cfg(feature = "preempt")]
    preempt_disable_count: AtomicUsize,
    /// 存储当前线程的TrapContext
    pub trap_frame: UnsafeCell<TrapFrame>,
    kstack: Option<TaskStack>,
    ctx: UnsafeCell<TaskContext>,
    time: UnsafeCell<TimeStat>,
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

impl TaskId {
    pub fn new() -> Self {
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
    /// 清空计数器，为了给单元测试使用
    /// 保留了gc, 主调度，内核进程
    pub fn clear() {
        ID_COUNTER.store(3, Ordering::Relaxed);
    }
}

impl From<u8> for TaskState {
    fn from(state: u8) -> Self {
        match state {
            1 => Self::Running,
            2 => Self::Ready,
            3 => Self::Blocked,
            4 => Self::Exited,
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

    pub fn get_process_id(&self) -> u64 {
        self.process_id
    }
}

// private methods
impl TaskInner {
    fn new_common(
        id: TaskId,
        name: &'static str,
        process_id: u64,
        page_table_token: usize,
    ) -> Self {
        Self {
            id,
            name,
            is_idle: false,
            is_init: false,
            process_id,
            exit_code: AtomicI32::new(0),
            page_table_token,
            is_leader: AtomicBool::new(false),
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            trap_frame: UnsafeCell::new(TrapFrame::default()),
            kstack: None,
            ctx: UnsafeCell::new(TaskContext::new()),
            time: UnsafeCell::new(TimeStat::new()),
        }
    }

    pub fn new<F>(
        entry: F,
        name: &'static str,
        stack_size: usize,
        process_id: u64,
        page_table_token: usize,
    ) -> AxTaskRef
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common(TaskId::new(), name, process_id, page_table_token);
        t.set_leader(true);
        let kstack = TaskStack::alloc(align_up_4k(stack_size));
        t.entry = Some(Box::into_raw(Box::new(entry)));
        t.ctx.get_mut().init(task_entry as usize, kstack.top());
        t.kstack = Some(kstack);
        if name == "idle" {
            t.is_idle = true;
        }
        Arc::new(AxTask::new(t))
    }

    pub(crate) fn new_init(name: &'static str) -> AxTaskRef {
        // init_task does not change PC and SP, so `entry` and `kstack` fields are not used.
        let mut t = Self::new_common(TaskId::new(), name, KERNEL_PROCESS_ID, 0);
        t.is_init = true;
        if name == "idle" {
            t.is_idle = true;
        }
        Arc::new(AxTask::new(t))
    }

    /// 获取内核栈栈顶
    #[inline]
    pub fn get_kernel_stack_top(&self) -> Option<usize> {
        if let Some(kstack) = &self.kstack {
            return Some(kstack.top().as_usize());
        }
        None
    }

    /// 获取内核栈的第一个trap上下文
    #[inline]
    pub fn get_first_trap_frame(&self) -> *mut TrapFrame {
        if let Some(kstack) = &self.kstack {
            return kstack.get_first_trap_frame();
        }
        unreachable!("get_first_trap_frame: kstack is None");
    }

    pub fn set_leader(&self, is_lead: bool) {
        self.is_leader.store(is_lead, Ordering::Release);
    }

    /// 设置Trap上下文
    pub fn set_trap_context(&self, trap_frame: TrapFrame) {
        let now_trap_frame = self.trap_frame.get();
        unsafe {
            *now_trap_frame = trap_frame;
        }
    }

    /// 将trap上下文直接写入到内核栈上
    /// 注意此时保持sp不变
    /// 返回值为压入了trap之后的内核栈的栈顶，可以用于多层trap压入
    pub fn set_trap_in_kernel_stack(&self) -> usize {
        let trap_frame_size = core::mem::size_of::<TrapFrame>();
        let frame_address = self.trap_frame.get();
        let kernel_base = self.get_kernel_stack_top().unwrap() - trap_frame_size;
        unsafe {
            __copy(frame_address, kernel_base);
        }
        kernel_base
    }

    #[inline]
    pub(crate) fn state(&self) -> TaskState {
        self.state.load(Ordering::Acquire).into()
    }

    #[inline]
    pub(crate) fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release)
    }

    #[inline]
    pub(crate) fn set_exit_code(&self, exit_code: i32) {
        self.exit_code.store(exit_code, Ordering::Release)
    }

    #[inline]
    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn is_running(&self) -> bool {
        matches!(self.state(), TaskState::Running)
    }

    #[inline]
    pub(crate) fn is_ready(&self) -> bool {
        matches!(self.state(), TaskState::Ready)
    }

    #[inline]
    pub(crate) fn is_blocked(&self) -> bool {
        matches!(self.state(), TaskState::Blocked)
    }

    #[inline]
    pub(crate) const fn is_init(&self) -> bool {
        self.is_init
    }

    #[inline]
    pub(crate) const fn is_idle(&self) -> bool {
        self.is_idle
    }

    #[inline]
    pub(crate) fn in_wait_queue(&self) -> bool {
        self.in_wait_queue.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn set_in_wait_queue(&self, in_wait_queue: bool) {
        self.in_wait_queue.store(in_wait_queue, Ordering::Release);
    }

    #[inline]
    pub fn time_stat_from_user_to_kernel(&self) {
        let time = self.time.get();
        unsafe {
            (*time).into_kernel_mode();
        }
    }

    #[inline]
    pub fn time_stat_from_kernel_to_user(&self) {
        let time = self.time.get();
        unsafe {
            (*time).into_user_mode();
        }
    }

    #[inline]
    pub fn time_stat_when_switch_from(&self) {
        let time = self.time.get();
        unsafe {
            (*time).swtich_from();
        }
    }

    #[inline]
    pub fn time_stat_when_switch_to(&self) {
        let time = self.time.get();
        unsafe {
            (*time).switch_to();
        }
    }

    #[inline]
    /// 将时间转为秒与微妙的形式输出，方便进行sys_time
    /// (用户态秒，用户态微妙，内核态秒，内核态微妙)
    pub fn time_stat_output(&self) -> (usize, usize, usize, usize) {
        let time = self.time.get();
        unsafe { (*time).output_as_us() }
    }

    #[inline]
    /// 重置统计时间
    pub fn time_stat_clear(&self) {
        let time = self.time.get();
        unsafe {
            (*time).clear();
        }
    }

    #[inline]
    pub(crate) fn in_timer_list(&self) -> bool {
        self.in_timer_list.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_state_running(&self) {
        self.state
            .store(TaskState::Running as u8, Ordering::Release);
    }

    #[inline]
    pub(crate) fn set_in_timer_list(&self, in_timer_list: bool) {
        self.in_timer_list.store(in_timer_list, Ordering::Release);
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn set_preempt_pending(&self, pending: bool) {
        self.need_resched.store(pending, Ordering::Release)
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn can_preempt(&self, current_disable_count: usize) -> bool {
        self.preempt_disable_count.load(Ordering::Acquire) == current_disable_count
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn disable_preempt(&self) {
        self.preempt_disable_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn enable_preempt(&self, resched: bool) {
        if self.preempt_disable_count.fetch_sub(1, Ordering::Relaxed) == 1 && resched {
            // If current task is pending to be preempted, do rescheduling.
            Self::current_check_preempt_pending();
        }
    }

    #[cfg(feature = "preempt")]
    fn current_check_preempt_pending() {
        let curr = crate::current();
        if curr.need_resched.load(Ordering::Acquire) && curr.can_preempt(0) {
            let mut rq = crate::RUN_QUEUE.lock();
            if curr.need_resched.load(Ordering::Acquire) {
                rq.resched();
            }
        }
    }

    #[inline]
    pub(crate) const fn page_table_token(&self) -> usize {
        self.page_table_token
    }

    #[inline]
    pub const unsafe fn ctx_mut_ptr(&self) -> *mut TaskContext {
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
    /// top是内核栈的最高地址
    /// 获取栈底，也即刚初始化时的栈顶
    pub const fn top(&self) -> VirtAddr {
        unsafe { core::mem::transmute(self.ptr.as_ptr().add(self.layout.size())) }
    }
    /// 获取内核栈第一个压入的trap上下文，防止出现内核trap嵌套
    pub fn get_first_trap_frame(&self) -> *mut TrapFrame {
        (self.top().as_usize() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame
    }
}

impl Drop for TaskStack {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

use core::mem::ManuallyDrop;

pub struct CurrentTask(ManuallyDrop<AxTaskRef>);

impl CurrentTask {
    pub(crate) fn try_get() -> Option<Self> {
        let ptr: *const super::AxTask = axhal::cpu::current_task_ptr();
        if !ptr.is_null() {
            Some(Self(unsafe { ManuallyDrop::new(AxTaskRef::from_raw(ptr)) }))
        } else {
            None
        }
    }

    pub(crate) fn get() -> Self {
        Self::try_get().expect("current task is uninitialized")
    }

    pub(crate) fn as_task_ref(&self) -> &AxTaskRef {
        &self.0
    }

    pub(crate) fn clone(&self) -> AxTaskRef {
        self.0.deref().clone()
    }

    pub(crate) fn ptr_eq(&self, other: &AxTaskRef) -> bool {
        Arc::ptr_eq(&self.0, other)
    }

    pub unsafe fn init_current(init_task: AxTaskRef) {
        let ptr = Arc::into_raw(init_task);
        axhal::cpu::set_current_task_ptr(ptr);
    }

    pub unsafe fn set_current(prev: Self, next: AxTaskRef) {
        let Self(arc) = prev;
        ManuallyDrop::into_inner(arc); // `call Arc::drop()` to decrease prev task reference count.
        let ptr = Arc::into_raw(next);
        axhal::cpu::set_current_task_ptr(ptr);
    }
}

impl Deref for CurrentTask {
    type Target = TaskInner;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// 初始化主进程的trap上下文
#[no_mangle]
// #[cfg(feature = "user")]
fn first_into_user(kernel_sp: usize, frame_base: usize) -> ! {
    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    unsafe {
        asm::sfence_vma_all();
        core::arch::asm!(
            r"
            mv      sp, {frame_base}
            LDR     gp, sp, 2                   // load user gp and tp
            LDR     t0, sp, 3
            mv      t1, {kernel_base}
            STR     tp, t1, 3                   // save supervisor tp，注意是存储到内核栈上而不是sp中
            mv      tp, t0                      // tp：线程指针
            csrw    sscratch, {kernel_sp}       // put supervisor sp to scratch
            LDR     t0, sp, 31
            LDR     t1, sp, 32
            csrw    sepc, t0
            csrw    sstatus, t1
            POP_GENERAL_REGS
            LDR     sp, sp, 1
            sret
        ",
            frame_base = in(reg) frame_base,
            kernel_sp = in(reg) kernel_sp,
            kernel_base = in(reg) kernel_base,
        );
    };
    core::panic!("already in user mode!")
}

#[no_mangle]
/// 本线程将会执行的函数
extern "C" fn task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { crate::RUN_QUEUE.force_unlock() };
    axhal::arch::enable_irqs();
    let task: CurrentTask = crate::current();
    if let Some(entry) = task.entry {
        if task.process_id == KERNEL_PROCESS_ID {
            // 是初始调度进程，直接执行即可
            unsafe { Box::from_raw(entry)() };
            // 继续执行对应的函数
        } else {
            // 需要通过切换特权级进入到对应的应用程序
            let kernel_sp = task.get_kernel_stack_top().unwrap();
            let frame_address = task.trap_frame.get();
            // 切换页表已经在switch实现了
            first_into_user(kernel_sp, frame_address as usize);
            // 问题：能否回来?
        }
    }
    // 任务执行完成，释放自我
    unreachable!("test!");
    // crate::exit(0);
}
