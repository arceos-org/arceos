use alloc::{boxed::Box, string::String, sync::Arc};
#[cfg(feature = "monolithic")]
use axconfig::SMP;

#[cfg(feature = "monolithic")]
use axhal::KERNEL_PROCESS_ID;

use core::ops::Deref;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicU8, Ordering};
use core::{alloc::Layout, cell::UnsafeCell, fmt, ptr::NonNull};

#[cfg(feature = "preempt")]
use core::sync::atomic::AtomicUsize;

#[cfg(feature = "tls")]
use axhal::tls::TlsArea;

use axhal::arch::TaskContext;
use memory_addr::{align_up_4k, VirtAddr};

#[cfg(feature = "monolithic")]
core::arch::global_asm!(include_str!("copy.S"));

#[cfg(feature = "monolithic")]
use axhal::arch::TrapFrame;

use crate::stat::TimeStat;

use crate::{AxRunQueue, AxTask, AxTaskRef, WaitQueue};

/// A unique identifier for a thread.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(u64);

/// The possible states of a task.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TaskState {
    Running = 1,
    Ready = 2,
    Blocked = 3,
    Exited = 4,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum SchedPolicy {
    SCHED_OTHER = 0,
    SCHED_FIFO = 1,
    SCHED_RR = 2,
    SCHED_BATCH = 3,
    SCHED_IDLE = 5,
    SCHED_UNKNOWN,
}

impl From<usize> for SchedPolicy {
    #[inline]
    fn from(policy: usize) -> Self {
        match policy {
            0 => SchedPolicy::SCHED_OTHER,
            1 => SchedPolicy::SCHED_FIFO,
            2 => SchedPolicy::SCHED_RR,
            3 => SchedPolicy::SCHED_BATCH,
            5 => SchedPolicy::SCHED_IDLE,
            _ => SchedPolicy::SCHED_UNKNOWN,
        }
    }
}

impl From<SchedPolicy> for isize {
    #[inline]
    fn from(policy: SchedPolicy) -> Self {
        match policy {
            SchedPolicy::SCHED_OTHER => 0,
            SchedPolicy::SCHED_FIFO => 1,
            SchedPolicy::SCHED_RR => 2,
            SchedPolicy::SCHED_BATCH => 3,
            SchedPolicy::SCHED_IDLE => 5,
            SchedPolicy::SCHED_UNKNOWN => -1,
        }
    }
}
#[derive(Clone, Copy)]
pub struct SchedStatus {
    pub policy: SchedPolicy,
    pub priority: usize,
}
/// The inner task structure.
pub struct TaskInner {
    id: TaskId,
    name: String,
    is_idle: bool,
    is_init: bool,

    entry: Option<*mut dyn FnOnce()>,
    state: AtomicU8,

    in_wait_queue: AtomicBool,
    #[cfg(feature = "irq")]
    in_timer_list: AtomicBool,

    #[cfg(feature = "preempt")]
    need_resched: AtomicBool,
    #[cfg(feature = "preempt")]
    preempt_disable_count: AtomicUsize,

    exit_code: AtomicI32,
    wait_for_exit: WaitQueue,

    kstack: Option<TaskStack>,
    ctx: UnsafeCell<TaskContext>,

    #[cfg(feature = "tls")]
    tls: TlsArea,

    #[cfg(feature = "monolithic")]
    process_id: AtomicU64,

    #[cfg(feature = "monolithic")]
    /// 是否是所属进程下的主线程
    is_leader: AtomicBool,

    #[cfg(feature = "monolithic")]
    /// 初始化的trap上下文
    pub trap_frame: UnsafeCell<TrapFrame>,

    #[cfg(feature = "monolithic")]
    pub page_table_token: usize,

    #[cfg(feature = "monolithic")]
    set_child_tid: AtomicU64,

    #[cfg(feature = "monolithic")]
    clear_child_tid: AtomicU64,

    // 时间统计, 无论是否为宏内核架构都可能被使用到
    #[allow(unused)]
    time: UnsafeCell<TimeStat>,

    #[cfg(feature = "monolithic")]
    pub cpu_set: AtomicU64,

    #[cfg(feature = "signal")]
    /// 退出时是否向父进程发送SIG_CHILD
    pub send_sigchld_when_exit: bool,

    #[cfg(feature = "monolithic")]
    pub sched_status: UnsafeCell<SchedStatus>,
}
static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
impl TaskId {
    pub fn new() -> Self {
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Convert the task ID to a `u64`.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "monolithic")]
    /// 清空计数器，为了给单元测试使用
    /// 保留了gc, 主调度，内核进程
    pub fn clear() {
        ID_COUNTER.store(5, Ordering::Relaxed);
    }
}

impl From<u8> for TaskState {
    #[inline]
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
    /// Gets the ID of the task.
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Gets the name of the task.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a combined string of the task ID and name.
    pub fn id_name(&self) -> alloc::string::String {
        alloc::format!("Task({}, {:?})", self.id.as_u64(), self.name)
    }

    /// Wait for the task to exit, and return the exit code.
    ///
    /// It will return immediately if the task has already exited (but not dropped).
    pub fn join(&self) -> Option<i32> {
        self.wait_for_exit
            .wait_until(|| self.state() == TaskState::Exited);
        Some(self.exit_code.load(Ordering::Acquire))
    }

    /// 获取内核栈栈顶
    #[inline]
    pub fn get_kernel_stack_top(&self) -> Option<usize> {
        if let Some(kstack) = &self.kstack {
            return Some(kstack.top().as_usize());
        }
        None
    }
}

#[cfg(feature = "monolithic")]
impl TaskInner {
    pub fn set_child_tid(&self, tid: usize) {
        self.set_child_tid.store(tid as u64, Ordering::Release)
    }

    pub fn set_clear_child_tid(&self, tid: usize) {
        self.clear_child_tid.store(tid as u64, Ordering::Release)
    }

    pub fn get_clear_child_tid(&self) -> usize {
        self.clear_child_tid.load(Ordering::Acquire) as usize
    }

    #[inline]
    pub fn get_page_table_token(&self) -> usize {
        self.page_table_token
    }

    #[inline]
    pub fn time_stat_from_user_to_kernel(&self) {
        let time = self.time.get();
        unsafe {
            (*time).into_kernel_mode(self.id.as_u64() as isize);
        }
    }

    #[inline]
    pub fn time_stat_from_kernel_to_user(&self) {
        let time = self.time.get();
        unsafe {
            (*time).into_user_mode(self.id.as_u64() as isize);
        }
    }

    #[inline]
    pub fn time_stat_when_switch_from(&self) {
        let time = self.time.get();
        unsafe {
            (*time).swtich_from(self.id.as_u64() as isize);
        }
    }

    #[inline]
    pub fn time_stat_when_switch_to(&self) {
        let time = self.time.get();
        unsafe {
            (*time).switch_to(self.id.as_u64() as isize);
        }
    }

    #[inline]
    /// 将内核统计的运行时时间转为秒与微妙的形式输出，方便进行sys_time
    /// (用户态秒，用户态微妙，内核态秒，内核态微妙)
    pub fn time_stat_output(&self) -> (usize, usize, usize, usize) {
        let time = self.time.get();
        unsafe { (*time).output_as_us() }
    }

    #[inline]
    /// 输出计时器信息
    /// (计时器周期，当前计时器剩余时间)
    /// 单位为us
    pub fn timer_output(&self) -> (usize, usize) {
        let time = self.time.get();
        unsafe { (*time).output_timer_as_us() }
    }

    #[inline]
    /// 设置计时器信息
    ///
    /// 若type不为None则返回成功
    pub fn set_timer(
        &self,
        timer_interval_ns: usize,
        timer_remained_ns: usize,
        timer_type: usize,
    ) -> bool {
        let time = self.time.get();
        unsafe { (*time).set_timer(timer_interval_ns, timer_remained_ns, timer_type) }
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
    pub fn get_process_id(&self) -> u64 {
        self.process_id.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_process_id(&self, process_id: u64) {
        self.process_id.store(process_id, Ordering::Release);
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

    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::Acquire)
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
    pub fn set_trap_in_kernel_stack(&self) {
        extern "C" {
            pub fn __copy(frame_address: *mut TrapFrame, kernel_base: usize);
        }
        let trap_frame_size = core::mem::size_of::<TrapFrame>();
        let frame_address = self.trap_frame.get();
        let kernel_base = self.get_kernel_stack_top().unwrap() - trap_frame_size;
        unsafe {
            __copy(frame_address, kernel_base);
        }
    }
    /// 设置CPU set，其中set_size为bytes长度
    pub fn set_cpu_set(&self, mask: usize, set_size: usize) {
        let len = if set_size * 4 > SMP {
            SMP
        } else {
            set_size * 4
        };
        let now_mask = mask & 1 << ((len) - 1);
        self.cpu_set.store(now_mask as u64, Ordering::Release)
    }

    pub fn get_cpu_set(&self) -> usize {
        self.cpu_set.load(Ordering::Acquire) as usize
    }

    pub fn set_sched_status(&self, status: SchedStatus) {
        let prev_status = self.sched_status.get();
        unsafe {
            *prev_status = status;
        }
    }

    pub fn get_sched_status(&self) -> SchedStatus {
        let status = self.sched_status.get();
        unsafe { *status }
    }

    #[cfg(feature = "signal")]
    pub fn get_sig_child(&self) -> bool {
        self.send_sigchld_when_exit
    }

    #[cfg(feature = "signal")]
    pub fn set_sig_child(&mut self, sig_child: bool) {
        self.send_sigchld_when_exit = sig_child;
    }
}

// private methods
impl TaskInner {
    fn new_common(id: TaskId, name: String) -> Self {
        Self {
            id,
            name,
            is_idle: false,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            #[cfg(feature = "irq")]
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: None,
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::alloc(),

            time: UnsafeCell::new(TimeStat::new()),

            #[cfg(feature = "monolithic")]
            process_id: AtomicU64::new(KERNEL_PROCESS_ID),

            #[cfg(feature = "monolithic")]
            is_leader: AtomicBool::new(false),

            #[cfg(feature = "monolithic")]
            // 初始化的trap上下文
            trap_frame: UnsafeCell::new(TrapFrame::default()),

            #[cfg(feature = "monolithic")]
            page_table_token: 0,

            #[cfg(feature = "monolithic")]
            set_child_tid: AtomicU64::new(0),

            #[cfg(feature = "monolithic")]
            clear_child_tid: AtomicU64::new(0),

            #[cfg(feature = "monolithic")]
             // 一开始默认都可以运行在每个CPU上
            cpu_set: AtomicU64::new(((1 << SMP) - 1) as u64),

            #[cfg(feature = "monolithic")]
            sched_status: UnsafeCell::new(SchedStatus {
                policy: SchedPolicy::SCHED_FIFO,
                priority: 1,
            }),

            #[cfg(feature = "signal")]
            send_sigchld_when_exit: false,
        }
    }

    /// Create a new task with the given entry function and stack size.
    pub fn new<F>(
        entry: F,
        name: String,
        stack_size: usize,
        #[cfg(feature = "monolithic")] process_id: u64,
        #[cfg(feature = "monolithic")] page_table_token: usize,
        #[cfg(feature = "signal")] sig_child: bool,
    ) -> AxTaskRef
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common(TaskId::new(), name);
        debug!("new task: {}", t.id_name());
        let kstack = TaskStack::alloc(align_up_4k(stack_size));

        #[cfg(feature = "tls")]
        let tls = VirtAddr::from(t.tls.tls_ptr() as usize);
        #[cfg(not(feature = "tls"))]
        let tls = VirtAddr::from(0);
        t.entry = Some(Box::into_raw(Box::new(entry)));

        #[cfg(feature = "signal")]
        t.set_sig_child(sig_child);

        #[cfg(feature = "monolithic")]
        {
            t.process_id.store(process_id, Ordering::Release);

            t.page_table_token = page_table_token;

            // 需要修改ctx存储的栈地址，否则内核trap上下文会被修改
            t.ctx.get_mut().init(
                task_entry as usize,
                kstack.top() - core::mem::size_of::<TrapFrame>(),
                tls,
            );
        }

        #[cfg(not(feature = "monolithic"))]
        t.ctx.get_mut().init(task_entry as usize, kstack.top(), tls);

        t.kstack = Some(kstack);
        if t.name == "idle" {
            t.is_idle = true;
        }
        Arc::new(AxTask::new(t))
    }

    /// Creates an "init task" using the current CPU states, to use as the
    /// current task.
    ///
    /// As it is the current task, no other task can switch to it until it
    /// switches out.
    ///
    /// And there is no need to set the `entry`, `kstack` or `tls` fields, as
    /// they will be filled automatically when the task is switches out.
    pub(crate) fn new_init(name: String) -> AxTaskRef {
        let mut t = Self::new_common(TaskId::new(), name);
        t.is_init = true;
        if t.name == "idle" {
            t.is_idle = true;
        }
        Arc::new(AxTask::new(t))
    }

    #[inline]
    pub fn state(&self) -> TaskState {
        self.state.load(Ordering::Acquire).into()
    }

    #[inline]
    pub fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release)
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
    #[cfg(feature = "irq")]
    pub(crate) fn in_timer_list(&self) -> bool {
        self.in_timer_list.load(Ordering::Acquire)
    }

    #[inline]
    #[cfg(feature = "irq")]
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
                rq.preempt_resched();
            }
        }
    }

    pub(crate) fn notify_exit(&self, exit_code: i32, rq: &mut AxRunQueue) {
        self.exit_code.store(exit_code, Ordering::Release);
        self.wait_for_exit.notify_all_locked(false, rq);
    }

    #[inline]
    pub(crate) const unsafe fn ctx_mut_ptr(&self) -> *mut TaskContext {
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

    #[cfg(feature = "monolithic")]
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

/// A wrapper of [`AxTaskRef`] as the current task.
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

    /// Converts [`CurrentTask`] to [`AxTaskRef`].
    pub fn as_task_ref(&self) -> &AxTaskRef {
        &self.0
    }

    pub(crate) fn clone(&self) -> AxTaskRef {
        self.0.deref().clone()
    }

    pub(crate) fn ptr_eq(&self, other: &AxTaskRef) -> bool {
        Arc::ptr_eq(&self.0, other)
    }

    pub(crate) unsafe fn init_current(init_task: AxTaskRef) {
        #[cfg(feature = "tls")]
        axhal::arch::write_thread_pointer(init_task.tls.tls_ptr() as usize);
        let ptr = Arc::into_raw(init_task);
        axhal::cpu::set_current_task_ptr(ptr);
    }

    pub(crate) unsafe fn set_current(prev: Self, next: AxTaskRef) {
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

#[no_mangle]
#[cfg(feature = "monolithic")]
/// 手动进入用户态
///
/// 1. 将对应trap上下文压入内核栈
/// 2. 返回用户态
///
/// args：
///
/// 1. kernel_sp：内核栈顶
///
/// 2. frame_base：对应即将压入内核栈的trap上下文的地址
pub fn first_into_user(kernel_sp: usize, frame_base: usize) -> ! {
    use axhal::arch::disable_irqs;

    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    // 在保证将寄存器都存储好之后，再开启中断
    // 否则此时会因为写入csr寄存器过程中出现中断，导致出现异常
    disable_irqs();
    // 在内核态中，tp寄存器存储的是当前任务的CPU ID
    // 而当从内核态进入到用户态时，会将tp寄存器的值先存储在内核栈上，即把该任务对应的CPU ID存储在内核栈上
    // 然后将tp寄存器的值改为对应线程的tls指针的值
    // 因此在用户态中，tp寄存器存储的值是线程的tls指针的值
    // 而当从用户态进入到内核态时，会先将内核栈上的值读取到某一个中间寄存器t0中，然后将tp的值存入内核栈
    // 然后再将t0的值赋给tp，因此此时tp的值是当前任务的CPU ID
    // 对应实现在axhal/src/arch/riscv/trap.S中
    unsafe {
        riscv::asm::sfence_vma_all();
        core::arch::asm!(
            r"
            mv      sp, {frame_base}
            .short  0x2432                      // fld fs0,264(sp)
            .short  0x24d2                      // fld fs1,272(sp)
            mv      t1, {kernel_base}
            LDR     t0, sp, 2
            STR     gp, t1, 2
            mv      gp, t0
            LDR     t0, sp, 3
            STR     tp, t1, 3                   // save supervisor tp，注意是存储到内核栈上而不是sp中，此时存储的应该是当前运行的CPU的ID
            mv      tp, t0                      // tp：本来存储的是CPU ID，在这个时候变成了对应线程的TLS 指针
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

extern "C" fn task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { crate::RUN_QUEUE.force_unlock() };
    #[cfg(feature = "irq")]
    axhal::arch::enable_irqs();
    let task = crate::current();
    if let Some(entry) = task.entry {
        cfg_if::cfg_if! {
            if #[cfg(feature = "monolithic")] {
                use axhal::KERNEL_PROCESS_ID;
                if task.get_process_id() == KERNEL_PROCESS_ID {
                    // 是初始调度进程，直接执行即可
                    unsafe { Box::from_raw(entry)() };
                    // 继续执行对应的函数
                } else {
                    // 需要通过切换特权级进入到对应的应用程序
                    let kernel_sp = task.get_kernel_stack_top().unwrap();
                    let frame_address = task.get_first_trap_frame();
                    // 切换页表已经在switch实现了
                    // 记得更新时间
                    task.time_stat_from_kernel_to_user();
                    first_into_user(kernel_sp, frame_address as usize);
                }
            }
            else {
                unsafe { Box::from_raw(entry)() };
            }

        }
    }
    // only for kernel task
    crate::exit(0);
}
