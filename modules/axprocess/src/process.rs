use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use axhal::arch::{write_page_table_root, TaskContext, TrapFrame};
use axlog::info;
use axmem::memory_set::USER_STACK_SIZE;
const KERNEL_STACK_SIZE: usize = 4096;
use crate::flags::{CloneFlags, WaitStatus};
use axmem::memory_set::{get_app_data, MemorySet};
use axtask::{
    current,
    task::{CurrentTask, TaskInner},
    yield_now, AxTaskRef, TaskId, IDLE_TASK, RUN_QUEUE,
};
use spinlock::SpinNoIrq;

use riscv::asm;
pub static PID2PC: SpinNoIrq<BTreeMap<u64, Arc<Process>>> = SpinNoIrq::new(BTreeMap::new());
pub const KERNEL_PROCESS_ID: u64 = 1;
/// 进程的的数据结构
pub struct Process {
    /// 进程的pid和初始化的线程的tid是一样的
    pub pid: u64,
    pub inner: SpinNoIrq<ProcessInner>,
}

pub struct ProcessInner {
    /// 父进程的进程号
    pub parent: u64,
    /// 子进程
    pub children: Vec<Arc<Process>>,
    /// 子任务
    pub tasks: Vec<AxTaskRef>,
    /// 地址空间，由于存在地址空间共享，因此设计为Arc类型
    pub memory_set: Arc<SpinNoIrq<MemorySet>>,
    /// 用户堆基址，任何时候堆顶都不能比这个值小，理论上讲是一个常量
    pub heap_bottom: usize,
    /// 当前用户堆的堆顶，不能小于基址，不能大于基址加堆的最大大小
    pub heap_top: usize,
    /// 进程状态
    pub is_zombie: bool,
    /// 退出状态码
    pub exit_code: i32,
}

impl ProcessInner {
    pub fn new(parent: u64, memory_set: Arc<SpinNoIrq<MemorySet>>, heap_bottom: usize) -> Self {
        Self {
            parent,
            children: Vec::new(),
            tasks: Vec::new(),
            memory_set,
            heap_bottom,
            heap_top: heap_bottom,
            is_zombie: false,
            exit_code: 0,
        }
    }
    pub fn get_page_table_token(&self) -> usize {
        self.memory_set.lock().page_table_token()
    }
}

impl Process {
    /// 根据name新建一个进程
    /// 此时应当是初始化的主进程，所以其父亲应当是内核进程
    pub fn new(name: &'static str) -> AxTaskRef {
        // 接下来是加载自己的内容
        // let mut page_table = copy_from_kernel_memory();
        // let (entry, user_stack_bottom) = load_from_elf(&mut page_table, get_app_data(name));
        let mut memory_set = MemorySet::new_from_kernel();
        let page_table_token = memory_set.page_table_token();
        let (entry, user_stack_bottom, heap_bottom) =
            MemorySet::from_elf(&mut memory_set, get_app_data(name));
        // 以这种方式建立的线程，不通过某一个具体的函数开始，而是通过地址来运行函数，所以entry不会被用到
        let new_process = Arc::new(Self {
            pid: TaskId::new().as_u64(),
            inner: SpinNoIrq::new(ProcessInner::new(
                KERNEL_PROCESS_ID,
                Arc::new(SpinNoIrq::new(memory_set)),
                heap_bottom,
            )),
        });
        // 记录该进程，防止被回收
        PID2PC
            .lock()
            .insert(new_process.pid, Arc::clone(&new_process));
        // 创立一个新的线程，初始化时进入
        let new_task = TaskInner::new(
            || {},
            name,
            KERNEL_STACK_SIZE,
            new_process.pid,
            page_table_token,
        );
        new_task.set_leader(true);
        // 初始化线程的trap上下文
        let new_trap_frame =
            TrapFrame::app_init_context(entry, user_stack_bottom + USER_STACK_SIZE);
        new_task.set_trap_context(new_trap_frame);
        // 设立父子关系
        let mut inner = new_process.inner.lock();
        inner.tasks.push(Arc::clone(&new_task));
        drop(inner);
        new_task.set_trap_in_kernel_stack();
        new_task
        // let kernel_sp = new_task.get_kernel_stack_top();
    }
    /// 将当前进程替换为指定的用户程序
    /// args为传入的参数
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        // 首先要处理原先进程的资源
        // 处理分配的页帧
        let mut inner = self.inner.lock();
        // 之后加入额外的东西之后再处理其他的包括信号等因素
        // 不是直接删除原有地址空间，否则构建成本较高。
        inner.memory_set.lock().unmap_user_areas();
        // 清空用户堆，重置堆顶
        unsafe {
            asm::sfence_vma_all();
        }
        let curr = current();
        // 再考虑手动结束其他所有的task
        let _ = inner
            .tasks
            .drain_filter(|task: &mut AxTaskRef| task.id() != curr.id())
            .map(|task| RUN_QUEUE.lock().remove_task(&task));
        // 当前任务被设置为主线程
        curr.set_leader(true);
        assert!(inner.tasks.len() == 1);
        let (entry, user_stack_bottom, heap_bottom) =
            MemorySet::from_elf(&mut inner.memory_set.lock(), elf_data);
        // 切换了地址空间， 需要切换token
        let page_table_token = if self.pid == KERNEL_PROCESS_ID {
            0
        } else {
            inner.memory_set.lock().page_table_token()
        };
        if page_table_token != 0 {
            // axhal::arch::write_page_table_root(page_table_token.into());
            unsafe { write_page_table_root(page_table_token.into()) };
        }
        // 重置用户堆
        inner.heap_bottom = heap_bottom;
        inner.heap_top = inner.heap_bottom;
        drop(inner);
        let mut user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        user_stack_top -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_stack_top;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| (argv_base + arg * core::mem::size_of::<usize>()) as *mut usize)
            .collect();
        unsafe {
            *argv[args.len()] = 0;
        }
        for i in 0..args.len() {
            user_stack_top -= args[i].len() + 1;
            unsafe {
                *argv[i] = user_stack_top;
            }
            let mut p = user_stack_top;
            for c in args[i].as_bytes() {
                unsafe {
                    *(p as *mut u8) = *c;
                }
                p += 1;
            }
            unsafe {
                *(p as *mut u8) = 0;
            }
        }
        // 对齐到4K
        user_stack_top -= user_stack_top % core::mem::size_of::<usize>();
        // user_stack_top = user_stack_top / PAGE_SIZE_4K * PAGE_SIZE_4K;
        let new_trap_frame = TrapFrame::app_init_context(entry, user_stack_top);
        curr.set_trap_context(new_trap_frame);
        let frame_address = curr.trap_frame.get() as usize;
        unsafe {
            // curr.trap_frame.get_mut().regs.a0 = args.len();
            // curr.trap_frame.get_mut().regs.a1 = argv_base;
            *((frame_address + 9 * core::mem::size_of::<usize>()) as *mut usize) = args.len();
            *((frame_address + 10 * core::mem::size_of::<usize>()) as *mut usize) = argv_base;
        }
        curr.set_trap_in_kernel_stack();
    }
    /// 实现简易的clone系统调用
    /// 返回值为新产生的任务的id
    pub fn clone_task(
        &self,
        flags: CloneFlags,
        stack: Option<usize>,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> u64 {
        let mut inner = self.inner.lock();
        // 是否共享虚拟地址空间
        let new_memory_set = if flags.contains(CloneFlags::CLONE_VM) {
            // 若是则直接共享指针即可
            Arc::clone(&inner.memory_set)
        } else {
            // 否则复制地址空间
            Arc::new(SpinNoIrq::new(MemorySet::new_from_task(
                &(inner.memory_set.lock()),
            )))
        };

        // 在生成新的进程前，需要决定其所属进程是谁
        let process_id = if flags.contains(CloneFlags::CLONE_THREAD) {
            // 当前clone生成的是线程，那么以self作为进程
            self.pid
        } else {
            // 新建一个进程，并且设计进程之间的父子关系
            TaskId::new().as_u64()
        };
        // 决定父进程是谁
        let parent_id = if flags.contains(CloneFlags::CLONE_PARENT) {
            // 创建兄弟关系，此时以self的父进程作为自己的父进程
            // 理论上不应该创建内核进程的兄弟进程，所以可以直接unwrap
            inner.parent
        } else {
            // 创建父子关系，此时以self作为父进程
            self.pid
        };
        // let new_process =
        let new_task = TaskInner::new(
            || {},
            "",
            KERNEL_STACK_SIZE,
            process_id,
            new_memory_set.lock().page_table_token(),
        );
        // 返回的值
        // 若创建的是进程，则返回进程的id
        // 若创建的是线程，则返回线程的id
        let mut return_id: u64 = 0;
        // 决定是创建线程还是进程
        if flags.contains(CloneFlags::CLONE_THREAD) {
            // 若创建的是进程，那么不用新建进程
            inner.tasks.push(Arc::clone(&new_task));
            return_id = new_task.id().as_u64();
        } else {
            // 若创建的是进程，那么需要新建进程
            // 由于地址空间是复制的，所以堆底的地址也一定相同
            let new_process = Arc::new(Self {
                pid: process_id,
                inner: SpinNoIrq::new(ProcessInner::new(
                    parent_id,
                    new_memory_set,
                    inner.heap_bottom,
                )),
            });
            // 记录该进程，防止被回收
            PID2PC.lock().insert(process_id, Arc::clone(&new_process));
            new_process.inner.lock().tasks.push(Arc::clone(&new_task));
            // 若是新建了进程，那么需要把进程的父子关系进行记录
            // info!("new process id:{}", new_process.pid);
            return_id = new_process.pid;
            inner.children.push(new_process);
        };
        drop(inner);
        if !flags.contains(CloneFlags::CLONE_THREAD) {
            new_task.set_leader(true);
        }
        let curr = current();
        let mut trap_frame = unsafe { *(curr.get_first_trap_frame()) };
        drop(curr);
        // 新开的进程/线程返回值为0
        trap_frame.regs.a0 = 0;
        if flags.contains(CloneFlags::CLONE_SETTLS) {
            trap_frame.regs.tp = tls;
        }
        // 设置用户栈
        // 若给定了用户栈，则使用给定的用户栈
        // 若没有给定用户栈，则使用当前用户栈
        // 没有给定用户栈的时候，只能是共享了地址空间，且原先调用clone的有用户栈，此时已经在之前的trap clone时复制了
        if let Some(stack) = stack {
            trap_frame.regs.sp = stack;
            axlog::info!(
                "New user stack: sepc:{:X}, stack:{:X}",
                trap_frame.sepc,
                trap_frame.regs.sp
            );
        }
        new_task.set_trap_context(trap_frame);
        new_task.set_trap_in_kernel_stack();
        RUN_QUEUE.lock().add_task(new_task);
        return_id
    }
    /// 若进程运行完成，则获取其返回码
    /// 若正在运行（可能上锁或没有上锁），则返回None
    fn get_code_if_exit(&self) -> Option<i32> {
        let inner = self.inner.try_lock()?;
        if inner.is_zombie {
            return Some(inner.exit_code);
        }
        None
    }
}

pub fn init_process() {
    // 内核的堆不重要，或者说当前未考虑内核堆的问题
    let kernel_process = Arc::new(Process {
        pid: TaskId::new().as_u64(),
        inner: SpinNoIrq::new(ProcessInner::new(
            0,
            Arc::new(SpinNoIrq::new(MemorySet::new_empty())),
            0,
        )),
    });
    axtask::init_scheduler();
    PID2PC
        .lock()
        .insert(kernel_process.pid, Arc::clone(&kernel_process));
    kernel_process.inner.lock().tasks.push(Arc::clone(unsafe {
        &IDLE_TASK.current_ref_raw().get_unchecked()
    }));
    let main_task = Process::new("helloworld");
    RUN_QUEUE.lock().add_task(main_task);
}

pub fn exit(exit_code: i32) -> isize {
    let curr = current();
    let is_leader = curr.is_leader();
    let process_id = curr.get_process_id();
    drop(curr);
    RUN_QUEUE.lock().exit_current(exit_code);
    // 若退出的是内核线程，就没有必要考虑后续了，否则此时调度队列重新调度的操作拿到进程这里来
    // 先进行资源的回收
    // 不可以回收内核任务
    if is_leader {
        assert!(process_id != 0);
        let pid2pc_inner = PID2PC.lock();
        let process = Arc::clone(&pid2pc_inner.get(&process_id).unwrap());
        drop(pid2pc_inner);
        let mut inner = process.inner.lock();
        inner.exit_code = exit_code;
        info!("process {} exit with code {}", process_id, exit_code);
        inner.is_zombie = true;
        {
            let pid2pc = PID2PC.lock();
            let kernel_process = Arc::clone(pid2pc.get(&KERNEL_PROCESS_ID).unwrap());
            drop(pid2pc);
            // 回收子进程到内核进程下
            for child in inner.children.iter() {
                child.inner.lock().parent = KERNEL_PROCESS_ID;
                kernel_process.inner.lock().children.push(Arc::clone(child));
            }
        }
        // 回收物理页帧
        inner.memory_set.lock().areas.clear();
        // 页表不用特意解除，因为整个对象都将被析构
        drop(inner);
    }
    // 当前的进程回收是比较简单的
    RUN_QUEUE.lock().resched_inner(false);
    exit_code as isize
}

/// 在当前进程找对应的子进程，并等待子进程结束
/// 若找到了则返回对应的pid
/// 否则返回一个状态
pub fn wait_pid(pid: isize, exit_code_ptr: *mut i32) -> Result<u64, WaitStatus> {
    // 获取当前进程
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    let mut inner = curr_process.inner.lock();
    let mut exit_task_id: usize = 0;
    let mut answer_id: u64 = 0;
    let mut answer_status = WaitStatus::NotExist;
    for (index, child) in inner.children.iter().enumerate() {
        if pid == -1 {
            // 任意一个进程结束都可以的
            answer_status = WaitStatus::Running;
            if let Some(exit_code) = child.get_code_if_exit() {
                answer_status = WaitStatus::Exited;
                exit_task_id = index;
                if !exit_code_ptr.is_null() {
                    unsafe {
                        // 因为没有切换页表，所以可以直接填写
                        *exit_code_ptr = exit_code;
                    }
                }
                answer_id = child.pid;
                break;
            }
        } else if child.pid == pid as u64 {
            // 找到了对应的进程
            if let Some(exit_code) = child.get_code_if_exit() {
                answer_status = WaitStatus::Exited;
                exit_task_id = index;
                if !exit_code_ptr.is_null() {
                    unsafe {
                        *exit_code_ptr = exit_code;
                    }
                }
                answer_id = child.pid;
            } else {
                answer_status = WaitStatus::Running;
            }
            break;
        }
    }
    // 若进程成功结束，需要将其从父进程的children中删除
    if answer_status == WaitStatus::Exited {
        inner.children.remove(exit_task_id as usize);
        return Ok(answer_id);
    }
    Err(answer_status)
}

/// 以进程作为中转调用task的yield
pub fn yield_now_task() {
    axtask::yield_now();
}

pub fn sleep_now_dur(dur: core::time::Duration) {
    axtask::sleep(dur);
}

pub fn current_task() -> CurrentTask {
    axtask::current()
}
