use alloc::string::ToString;
use alloc::vec;
use alloc::{collections::BTreeMap, format, string::String, sync::Arc, vec::Vec};
use axerrno::{AxError, AxResult};
use axfs::monolithic_fs::FileIO;
use axhal::arch::{write_page_table_root, TrapFrame};
use axhal::mem::phys_to_virt;
use axlog::debug;
use axtask::monolithic_task::task::TaskState;
use axtask::{AxTaskRef, TaskId};

const KERNEL_STACK_SIZE: usize = 0x40000;
const FD_LIMIT_ORIGIN: usize = 256;
use crate::fd_manager::FdManager;
use crate::flags::{CloneFlags, WaitStatus};
use crate::futex::{clear_wait, FutexRobustList};
use crate::loader::load_app;
use crate::send_signal_to_process;
use crate::signal::SignalModule;
use crate::stdin::{Stderr, Stdin, Stdout};
// use crate::test::finish_one_test;
use axmem::MemorySet;
use axtask::{
    current,
    monolithic_task::run_queue::{IDLE_TASK, RUN_QUEUE},
    monolithic_task::task::{CurrentTask, TaskInner},
};
use spinlock::SpinNoIrq;

use riscv::asm;
// pub(crate) static TID2TASK: SpinNoIrq<BTreeMap<TaskId, AxTaskRef>> =
//     SpinNoIrq::new(BTreeMap::new());
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
    // /// 文件描述符表
    // pub fd_table: Vec<Option<Arc<SpinNoIrq<dyn FileIO>>>>,
    // /// 文件描述符上限，由prlimit设置
    // pub fd_limit: usize,
    #[cfg(feature = "fs")]
    pub fd_manager: FdManager,
    /// 进程工作目录
    pub cwd: String,
    #[cfg(feature = "signal")]
    /// 信号处理模块    
    /// 第一维代表线程号，第二维代表线程对应的信号处理模块
    pub signal_module: BTreeMap<u64, SignalModule>,

    /// robust list存储模块
    /// 用来存储线程对共享变量的使用地址
    /// 具体使用交给了用户空间
    pub robust_list: BTreeMap<u64, FutexRobustList>,
}

impl ProcessInner {
    pub fn new(
        parent: u64,
        memory_set: Arc<SpinNoIrq<MemorySet>>,
        heap_bottom: usize,
        fd_table: Vec<Option<Arc<SpinNoIrq<dyn FileIO>>>>,
    ) -> Self {
        Self {
            parent,
            children: Vec::new(),
            tasks: Vec::new(),
            memory_set,
            heap_bottom,
            heap_top: heap_bottom,
            is_zombie: false,
            exit_code: 0,
            fd_manager: FdManager::new(fd_table, FD_LIMIT_ORIGIN),
            cwd: "/".to_string(), // 这里的工作目录是根目录
            #[cfg(feature = "signal")]
            signal_module: BTreeMap::new(),
            robust_list: BTreeMap::new(),
        }
    }
    pub fn get_page_table_token(&self) -> usize {
        self.memory_set.lock().page_table_token()
    }
    pub fn alloc_fd(&mut self) -> AxResult<usize> {
        for (i, fd) in self.fd_manager.fd_table.iter().enumerate() {
            if fd.is_none() {
                return Ok(i);
            }
        }
        if self.fd_manager.fd_table.len() >= self.fd_manager.limit {
            debug!("fd table is full");
            return Err(AxError::StorageFull);
        }
        self.fd_manager.fd_table.push(None);
        Ok(self.fd_manager.fd_table.len() - 1)
    }
    pub fn get_cwd(&self) -> String {
        self.cwd.clone()
    }
}

impl Process {
    /// 根据应用名寻找文件，作为初始化主进程启动
    /// 需要将其作为内核进程的子进程，方便回收
    pub fn new(args: Vec<String>) -> AxResult<AxTaskRef> {
        // 测例文件名
        let path = args[0].clone();
        let mut memory_set = MemorySet::new_with_kernel_mapped();
        let page_table_token = memory_set.page_table_token();
        if page_table_token != 0 {
            unsafe {
                write_page_table_root(page_table_token.into());
                riscv::register::sstatus::set_sum();
            };
        }

        let (entry, user_stack_bottom, heap_bottom) = load_app(path.clone(), args, &mut memory_set)
            .expect(format!("Failed to load app: {}", path).as_str());
        // 切换页表
        // 以这种方式建立的线程，不通过某一个具体的函数开始，而是通过地址来运行函数，所以entry不会被用到
        let new_process = Arc::new(Self {
            pid: TaskId::new().as_u64(),
            inner: SpinNoIrq::new(ProcessInner::new(
                KERNEL_PROCESS_ID,
                Arc::new(SpinNoIrq::new(memory_set)),
                heap_bottom.as_usize(),
                vec![
                    // 标准输入
                    Some(Arc::new(SpinNoIrq::new(Stdin))),
                    // 标准输出
                    Some(Arc::new(SpinNoIrq::new(Stdout))),
                    // 标准错误
                    Some(Arc::new(SpinNoIrq::new(Stderr))),
                    // // 工作目录, fd_table[3]固定用来存放工作目录
                    // Some(Arc::new(CurWorkDirDesc::new('/'.to_string()))),   // 这里的工作目录是根目录
                ],
            )),
        });

        // 创立一个新的线程，初始化时进入
        let new_task = TaskInner::new(
            || {},
            path,
            KERNEL_STACK_SIZE,
            new_process.pid,
            page_table_token,
            false,
        );
        new_task.set_leader(true);
        // 初始化线程的trap上下文
        // info!("new process: {}", new_process.pid);
        // info!("user_stack_bottom: {:x}", user_stack_bottom.as_usize());
        let new_trap_frame =
            TrapFrame::app_init_context(entry.as_usize(), user_stack_bottom.as_usize());

        new_task.set_trap_context(new_trap_frame);
        // info!("new task: {}", new_task.id().as_u64());
        // 设立父子关系
        let mut inner = new_process.inner.lock();
        inner
            .robust_list
            .insert(new_task.id().as_u64(), FutexRobustList::default());
        inner.tasks.push(Arc::clone(&new_task));
        // TID2TASK.lock().insert(new_task.id(), Arc::clone(&new_task));
        // 设置信号模块内容
        // info!("new task: {}", new_task.id().as_u64());
        inner
            .signal_module
            .insert(new_task.id().as_u64(), SignalModule::init_signal(None));

        PID2PC
            .lock()
            .insert(new_process.pid, Arc::clone(&new_process));

        drop(inner);
        // 将其作为内核进程的子进程
        match PID2PC.lock().get(&KERNEL_PROCESS_ID) {
            Some(kernel_process) => {
                kernel_process.inner.lock().children.push(new_process);
            }
            None => {
                return Err(AxError::NotFound);
            }
        }
        new_task.set_trap_in_kernel_stack();
        Ok(new_task)
        // let kernel_sp = new_task.get_kernel_stack_top();
    }
    /// 将当前进程替换为指定的用户程序
    /// args为传入的参数
    /// 任务的统计时间会被重置
    pub fn exec(&self, name: String, args: Vec<String>) {
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
        // 重置统计时间
        curr.time_stat_clear();
        assert!(inner.tasks.len() == 1);
        let args = if args.len() == 0 {
            vec![name.clone()]
        } else {
            args
        };
        let (entry, user_stack_bottom, heap_bottom) =
            load_app(name.clone(), args, &mut inner.memory_set.lock())
                .expect(format!("Failed to load app: {}", name).as_str());
        // 切换了地址空间， 需要切换token
        let page_table_token = if self.pid == KERNEL_PROCESS_ID {
            0
        } else {
            inner.memory_set.lock().page_table_token()
        };
        if page_table_token != 0 {
            // axhal::arch::write_page_table_root(page_table_token.into());
            unsafe {
                write_page_table_root(page_table_token.into());
                asm::sfence_vma_all();
            };
            // 清空用户堆，重置堆顶
        }
        // 重置用户堆
        inner.heap_bottom = heap_bottom.as_usize();
        inner.heap_top = inner.heap_bottom;

        // 重置robust list
        inner.robust_list.clear();
        inner
            .robust_list
            .insert(curr.id().as_u64(), FutexRobustList::default());

        // 重置信号处理模块
        // 此时只会留下一个线程
        inner.signal_module.clear();
        inner
            .signal_module
            .insert(curr.id().as_u64(), SignalModule::init_signal(None));
        drop(inner);
        // user_stack_top = user_stack_top / PAGE_SIZE_4K * PAGE_SIZE_4K;
        let new_trap_frame =
            TrapFrame::app_init_context(entry.as_usize(), user_stack_bottom.as_usize());
        curr.set_trap_context(new_trap_frame);
        curr.set_trap_in_kernel_stack();
    }
    /// 实现简易的clone系统调用
    /// 返回值为新产生的任务的id
    pub fn clone_task(
        &self,
        flags: CloneFlags,
        sig_child: bool,
        stack: Option<usize>,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AxResult<u64> {
        let mut inner = self.inner.lock();
        // 是否共享虚拟地址空间
        let new_memory_set = if flags.contains(CloneFlags::CLONE_VM) {
            Arc::clone(&inner.memory_set)
        } else {
            Arc::new(SpinNoIrq::new(MemorySet::clone(&inner.memory_set.lock())))
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
        let new_task = TaskInner::new(
            || {},
            String::new(),
            KERNEL_STACK_SIZE,
            process_id,
            new_memory_set.lock().page_table_token(),
            sig_child,
        );
        debug!("new task:{}", new_task.id().as_u64());
        // TID2TASK.lock().insert(new_task.id(), Arc::clone(&new_task));
        #[cfg(feature = "signal")]
        let new_handler = if flags.contains(CloneFlags::CLONE_SIGHAND) {
            // let curr_id = current_task().id().as_u64();
            inner
                .signal_module
                .get_mut(&current_task().id().as_u64())
                .unwrap()
                .signal_handler
                .clone()
        } else {
            // info!("curr_id: {:X}", (&curr_id as *const _ as usize));
            Arc::new(SpinNoIrq::new(
                inner
                    .signal_module
                    .get_mut(&current_task().id().as_u64())
                    .unwrap()
                    .signal_handler
                    .lock()
                    .clone(),
            ))
        };
        // 检查是否在父任务中写入当前新任务的tid
        if flags.contains(CloneFlags::CLONE_PARENT_SETTID) {
            if inner
                .memory_set
                .lock()
                .manual_alloc_for_lazy(ptid.into())
                .is_ok()
            {
                unsafe {
                    *(ptid as *mut i32) = new_task.id().as_u64() as i32;
                }
            }
        }
        // 若包含CLONE_CHILD_SETTID或者CLONE_CHILD_CLEARTID
        // 则需要把线程号写入到子线程地址空间中tid对应的地址中
        if flags.contains(CloneFlags::CLONE_CHILD_SETTID)
            || flags.contains(CloneFlags::CLONE_CHILD_CLEARTID)
        {
            if flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
                new_task.set_child_tid(ctid);
            }

            if flags.contains(CloneFlags::CLONE_CHILD_CLEARTID) {
                new_task.set_clear_child_tid(ctid);
            }

            if flags.contains(CloneFlags::CLONE_VM) {
                // 此时地址空间不会发生改变
                // 在当前地址空间下进行分配
                if inner
                    .memory_set
                    .lock()
                    .manual_alloc_for_lazy(ctid.into())
                    .is_ok()
                {
                    // 正常分配了地址
                    unsafe {
                        *(ctid as *mut i32) = if flags.contains(CloneFlags::CLONE_CHILD_SETTID) {
                            new_task.id().as_u64() as i32
                        } else {
                            0
                        }
                    }
                } else {
                    return Err(AxError::BadAddress);
                }
            } else {
                let mut vm = new_memory_set.lock();
                // 否则需要在新的地址空间中进行分配
                if vm.manual_alloc_for_lazy(ctid.into()).is_ok() {
                    // 此时token没有发生改变，所以不能直接解引用访问，需要手动查页表
                    if let Ok((phyaddr, _, _)) = vm.query(ctid.into()) {
                        let vaddr: usize = phys_to_virt(phyaddr).into();
                        // 注意：任何地址都是从free memory分配来的，那么在页表中，free memory一直在页表中，他们的虚拟地址和物理地址一直有偏移的映射关系
                        unsafe {
                            *(vaddr as *mut i32) = if flags.contains(CloneFlags::CLONE_CHILD_SETTID)
                            {
                                new_task.id().as_u64() as i32
                            } else {
                                0
                            }
                        }
                        drop(vm);
                    } else {
                        drop(vm);
                        return Err(AxError::BadAddress);
                    }
                } else {
                    drop(vm);
                    return Err(AxError::BadAddress);
                }
            }
        }
        // 返回的值
        // 若创建的是进程，则返回进程的id
        // 若创建的是线程，则返回线程的id
        let return_id: u64;
        // 决定是创建线程还是进程
        if flags.contains(CloneFlags::CLONE_THREAD) {
            // // 若创建的是线程，那么不用新建进程
            // info!("task len: {}", inner.tasks.len());
            // info!("task address :{:X}", (&new_task as *const _ as usize));
            // info!(
            //     "task address: {:X}",
            //     (&Arc::clone(&new_task)) as *const _ as usize
            // );
            inner.tasks.push(Arc::clone(&new_task));
            #[cfg(feature = "signal")]
            inner.signal_module.insert(
                new_task.id().as_u64(),
                SignalModule::init_signal(Some(new_handler)),
            );
            inner
                .robust_list
                .insert(new_task.id().as_u64(), FutexRobustList::default());
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
                    inner.fd_manager.fd_table.clone(),
                )),
            });
            // 记录该进程，防止被回收
            PID2PC.lock().insert(process_id, Arc::clone(&new_process));
            new_process.inner.lock().tasks.push(Arc::clone(&new_task));
            // 若是新建了进程，那么需要把进程的父子关系进行记录
            #[cfg(feature = "signal")]
            new_process.inner.lock().signal_module.insert(
                new_task.id().as_u64(),
                SignalModule::init_signal(Some(new_handler)),
            );

            new_process
                .inner
                .lock()
                .robust_list
                .insert(new_task.id().as_u64(), FutexRobustList::default());
            return_id = new_process.pid;
            inner.children.push(new_process);
        };
        drop(inner);
        if !flags.contains(CloneFlags::CLONE_THREAD) {
            new_task.set_leader(true);
        }
        let curr = current();
        // 复制原有的trap上下文
        let mut trap_frame = unsafe { *(curr.get_first_trap_frame()) }.clone();
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
            // info!(
            //     "New user stack: sepc:{:X}, stack:{:X}",
            //     trap_frame.sepc, trap_frame.regs.sp
            // );
        }
        new_task.set_trap_context(trap_frame);
        new_task.set_trap_in_kernel_stack();
        RUN_QUEUE.lock().add_task(new_task);
        Ok(return_id)
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

/// 初始化内核调度进程
pub fn init_kernel_process() {
    // 内核的堆不重要，或者说当前未考虑内核堆的问题
    let kernel_process = Arc::new(Process {
        pid: TaskId::new().as_u64(),
        inner: SpinNoIrq::new(ProcessInner::new(
            0,
            Arc::new(SpinNoIrq::new(MemorySet::new_empty())),
            0,
            vec![], // 内核进程不需要文件描述符
        )),
    });
    axtask::init_scheduler();
    PID2PC
        .lock()
        .insert(kernel_process.pid, Arc::clone(&kernel_process));
    kernel_process.inner.lock().tasks.push(Arc::clone(unsafe {
        &IDLE_TASK.current_ref_raw().get_unchecked()
    }));

    // TID2TASK.lock().insert(
    //     unsafe { IDLE_TASK.current_ref_raw().id() },
    //     Arc::clone(unsafe { &IDLE_TASK.current_ref_raw().get_unchecked() }),
    // );
}

/// 将进程转化为调度进程，此时会运行所有的测例文件
pub fn init_user_process() {
    let main_task = Process::new(["execve".to_string()].to_vec()).unwrap();
    RUN_QUEUE.lock().add_task(main_task);
}

/// 获取当前任务对应的进程
pub fn current_process() -> Arc<Process> {
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    curr_process
}

/// 退出当前任务
pub fn exit(exit_code: i32) -> ! {
    let process = current_process();
    let curr = current();
    let curr_id = curr.id();
    debug!(
        "curr_id: {}, exit_code: {}",
        curr_id.as_u64(),
        exit_code as i32
    );
    let is_leader = curr.is_leader();
    let process_id = curr.get_process_id();
    clear_wait(
        if curr.is_leader() {
            process_id
        } else {
            curr_id.as_u64()
        },
        curr.is_leader(),
    );
    // 检查这个任务是否有sig_child信号
    if curr.send_sigchld_when_exit || curr.is_leader() {
        let inner = process.inner.lock();
        let parent = inner.parent;
        drop(inner);
        if parent != KERNEL_PROCESS_ID {
            // 发送sigchild
            send_signal_to_process(parent as isize, 17).unwrap();
        }
    }
    let mut inner = process.inner.lock();
    // clear_child_tid 的值不为 0，则将这个用户地址处的值写为0
    let clear_child_tid = curr.get_clear_child_tid();
    if clear_child_tid != 0 {
        // 先确认是否在用户空间
        let mut memory_set = inner.memory_set.lock();
        if memory_set
            .manual_alloc_for_lazy(clear_child_tid.into())
            .is_ok()
        {
            unsafe {
                *(clear_child_tid as *mut i32) = 0;
            }
        }
        drop(memory_set);
    }

    // 记得删除任务
    // TID2TASK.lock().remove(&curr_id);

    // 若退出的是内核线程，就没有必要考虑后续了，否则此时调度队列重新调度的操作拿到进程这里来
    // 先进行资源的回收
    // 不可以回收内核任务
    if is_leader {
        assert!(process_id != 0);
        // WAIT_FOR_FUTEX.notify_all(false);
        drop(inner);
        loop {
            let inner = process.inner.lock();
            let mut all_exited = true;
            for task in inner.tasks.iter() {
                if !task.is_leader() && task.state() != TaskState::Exited {
                    all_exited = false;
                    break;
                }
            }
            drop(inner);
            if !all_exited {
                // info!("exit current: {}", curr_id.as_u64());
                yield_now_task();
            } else {
                break;
            }
        }

        let mut inner = process.inner.lock();
        inner.exit_code = exit_code;
        inner.is_zombie = true;
        RUN_QUEUE.lock().exit_current(exit_code);
        inner.tasks.clear();
        inner.signal_module.clear();
        inner.fd_manager.fd_table.clear();
        {
            let pid2pc = PID2PC.lock();
            let kernel_process = Arc::clone(pid2pc.get(&KERNEL_PROCESS_ID).unwrap());
            // 回收子进程到内核进程下
            // 销毁原进程的地址空间
            inner.memory_set = Arc::clone(&kernel_process.inner.lock().memory_set);
            for child in inner.children.iter() {
                child.inner.lock().parent = KERNEL_PROCESS_ID;
                kernel_process.inner.lock().children.push(Arc::clone(child));
            }

            drop(pid2pc);
        }
        // 不能在这里直接解除页表映射，因为存在地址空间共享机制
        // inner.memory_set.lock().unmap_user_areas();
        // assert!(Arc::strong_count(&inner.memory_set) )
        // 页表不用特意解除，因为整个对象都将被析构
        drop(inner);
        drop(process);
        let mut pid2pc = PID2PC.lock();
        pid2pc.remove(&process_id);
        drop(pid2pc);
        // 记录当前的测试结果
    } else {
        inner.signal_module.remove(&curr_id.as_u64());
        RUN_QUEUE.lock().exit_current(exit_code);
        drop(inner);
        drop(process);
    }

    RUN_QUEUE.lock().resched_inner(false);
    unreachable!("Unreachable in sys_exit!");
}

/// 在当前进程找对应的子进程，并等待子进程结束
/// 若找到了则返回对应的pid
/// 否则返回一个状态
pub fn wait_pid(pid: isize, exit_code_ptr: *mut i32) -> Result<u64, WaitStatus> {
    // 获取当前进程
    let curr_process = current_process();
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
                        *exit_code_ptr = exit_code << 8;
                        // 用于WEXITSTATUS设置编码
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

pub fn sleep_now_task(dur: core::time::Duration) {
    axtask::sleep(dur);
}

pub fn current_task() -> CurrentTask {
    axtask::current()
}

pub fn set_child_tid(tid: usize) {
    let curr = current_task();
    curr.set_clear_child_tid(tid);
}
