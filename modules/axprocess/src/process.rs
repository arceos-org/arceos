//! 规定进程控制块内容
extern crate alloc;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::{collections::BTreeMap, string::String};
use axerrno::{AxError, AxResult};
use axhal::arch::{write_page_table_root, TrapFrame};
use axhal::mem::phys_to_virt;
use axhal::KERNEL_PROCESS_ID;
use axlog::{debug, error};
use axmem::MemorySet;
use axsync::Mutex;
use axtask::{current, AxTaskRef, TaskId, TaskInner, RUN_QUEUE};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

use crate::flags::CloneFlags;
use crate::load_app;
pub static TID2TASK: Mutex<BTreeMap<u64, AxTaskRef>> = Mutex::new(BTreeMap::new());
pub static PID2PC: Mutex<BTreeMap<u64, Arc<Process>>> = Mutex::new(BTreeMap::new());
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

    /// 地址空间
    pub memory_set: Arc<Mutex<MemorySet>>,

    /// 用户堆基址，任何时候堆顶都不能比这个值小，理论上讲是一个常量
    pub heap_bottom: AtomicU64,

    /// 当前用户堆的堆顶，不能小于基址，不能大于基址加堆的最大大小
    pub heap_top: AtomicU64,
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

    pub fn get_heap_top(&self) -> u64 {
        self.heap_top.load(Ordering::Acquire)
    }

    pub fn set_heap_top(&self, top: u64) {
        self.heap_top.store(top, Ordering::Release)
    }

    pub fn get_heap_bottom(&self) -> u64 {
        self.heap_bottom.load(Ordering::Acquire)
    }

    pub fn set_heap_bottom(&self, bottom: u64) {
        self.heap_bottom.store(bottom, Ordering::Release)
    }

    /// 若进程运行完成，则获取其返回码
    /// 若正在运行（可能上锁或没有上锁），则返回None
    pub fn get_code_if_exit(&self) -> Option<i32> {
        if self.get_zombie() {
            return Some(self.get_exit_code());
        }
        None
    }
}

impl Process {
    pub fn new(pid: u64, parent: u64, memory_set: Arc<Mutex<MemorySet>>, heap_bottom: u64) -> Self {
        Self {
            pid,
            parent: AtomicU64::new(parent),
            children: Mutex::new(Vec::new()),
            tasks: Mutex::new(Vec::new()),
            is_zombie: AtomicBool::new(false),
            exit_code: AtomicI32::new(0),
            memory_set,
            heap_bottom: AtomicU64::new(heap_bottom),
            heap_top: AtomicU64::new(heap_bottom),
        }
    }
    /// 根据给定参数创建一个新的进程，作为应用程序初始进程
    pub fn init(args: Vec<String>) -> AxResult<AxTaskRef> {
        let path = args[0].clone();
        let mut memory_set = MemorySet::new_with_kernel_mapped();
        let page_table_token = memory_set.page_table_token();
        if page_table_token != 0 {
            unsafe {
                write_page_table_root(page_table_token.into());
                riscv::register::sstatus::set_sum();
            };
        }
        // 运行gcc程序时需要预先加载的环境变量
        let envs:Vec<String> = vec![
            "SHLVL=1".into(),
            "PATH=/usr/sbin:/usr/bin:/sbin:/bin".into(),
            "PWD=/".into(),
            "GCC_EXEC_PREFIX=/riscv64-linux-musl-native/bin/../lib/gcc/".into(),
            "COLLECT_GCC=./riscv64-linux-musl-native/bin/riscv64-linux-musl-gcc".into(),
            "COLLECT_LTO_WRAPPER=/riscv64-linux-musl-native/bin/../libexec/gcc/riscv64-linux-musl/11.2.1/lto-wrapper".into(),
            "COLLECT_GCC_OPTIONS='-march=rv64gc' '-mabi=lp64d' '-march=rv64imafdc' '-dumpdir' 'a.'".into(),
            "LIBRARY_PATH=/lib/".into(),
            "LD_LIBRARY_PATH=/lib/".into(),
        ];
        let (entry, user_stack_bottom, heap_bottom) =
            if let Ok(ans) = load_app(path.clone(), args, envs, &mut memory_set) {
                ans
            } else {
                error!("Failed to load app {}", path);
                return Err(AxError::NotFound);
            };
        let new_process = Arc::new(Self::new(
            TaskId::new().as_u64(),
            KERNEL_PROCESS_ID,
            Arc::new(Mutex::new(memory_set)),
            heap_bottom.as_usize() as u64,
        ));
        let new_task = TaskInner::new(
            || {},
            path,
            axconfig::TASK_STACK_SIZE,
            new_process.pid(),
            page_table_token,
        );
        TID2TASK
            .lock()
            .insert(new_task.id().as_u64(), Arc::clone(&new_task));
        new_task.set_leader(true);
        let new_trap_frame =
            TrapFrame::app_init_context(entry.as_usize(), user_stack_bottom.as_usize());
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
        RUN_QUEUE.lock().add_task(Arc::clone(&new_task));
        Ok(new_task)
    }
}

impl Process {
    /// 将当前进程替换为指定的用户程序
    /// args为传入的参数
    /// 任务的统计时间会被重置
    pub fn exec(&self, name: String, args: Vec<String>, envs: Vec<String>) -> AxResult<()> {
        // 首先要处理原先进程的资源
        // 处理分配的页帧
        // 之后加入额外的东西之后再处理其他的包括信号等因素
        // 不是直接删除原有地址空间，否则构建成本较高。
        self.memory_set.lock().unmap_user_areas();
        // 清空用户堆，重置堆顶
        unsafe {
            riscv::asm::sfence_vma_all();
        }

        // 关闭 `CLOEXEC` 的文件
        // inner.fd_manager.close_on_exec();

        let current_task = current();
        // 再考虑手动结束其他所有的task
        let mut tasks = self.tasks.lock();
        for _ in 0..tasks.len() {
            let task = tasks.pop().unwrap();
            if task.id() == current_task.id() {
                tasks.push(task);
            } else {
                TID2TASK.lock().remove(&task.id().as_u64());
                RUN_QUEUE.lock().remove_task(&task);
            }
        }
        // 当前任务被设置为主线程
        current_task.set_leader(true);
        // 重置统计时间
        current_task.time_stat_clear();
        assert!(self.tasks.lock().len() == 1);
        let args = if args.len() == 0 {
            vec![name.clone()]
        } else {
            args
        };
        let (entry, user_stack_bottom, heap_bottom) =
            if let Ok(ans) = load_app(name.clone(), args, envs, &mut self.memory_set.lock()) {
                ans
            } else {
                error!("Failed to load app {}", name);
                return Err(AxError::NotFound);
            };
        // 切换了地址空间， 需要切换token
        let page_table_token = if self.pid == KERNEL_PROCESS_ID {
            0
        } else {
            self.memory_set.lock().page_table_token()
        };
        if page_table_token != 0 {
            // axhal::arch::write_page_table_root(page_table_token.into());
            unsafe {
                write_page_table_root(page_table_token.into());
                riscv::asm::sfence_vma_all();
            };
            // 清空用户堆，重置堆顶
        }
        // 重置用户堆
        self.set_heap_bottom(heap_bottom.as_usize() as u64);
        self.set_heap_top(heap_bottom.as_usize() as u64);
        // // 重置robust list
        // inner.robust_list.clear();
        // inner
        //     .robust_list
        //     .insert(curr.id().as_u64(), FutexRobustList::default());

        // // 重置信号处理模块
        // // 此时只会留下一个线程
        // inner.signal_module.clear();
        // inner
        //     .signal_module
        //     .insert(curr.id().as_u64(), SignalModule::init_signal(None));
        // user_stack_top = user_stack_top / PAGE_SIZE_4K * PAGE_SIZE_4K;
        let new_trap_frame =
            TrapFrame::app_init_context(entry.as_usize(), user_stack_bottom.as_usize());
        current_task.set_trap_context(new_trap_frame);
        current_task.set_trap_in_kernel_stack();
        Ok(())
    }

    /// 实现简易的clone系统调用
    /// 返回值为新产生的任务的id
    pub fn clone_task(
        &self,
        flags: CloneFlags,
        _sig_child: bool,
        stack: Option<usize>,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AxResult<u64> {
        if self.tasks.lock().len() > 100 {
            // 任务过多，手动特判结束，用来作为QEMU内存不足的应对方法
            return Err(AxError::NoMemory);
        }

        // 是否共享虚拟地址空间
        let new_memory_set = if flags.contains(CloneFlags::CLONE_VM) {
            Arc::clone(&self.memory_set)
        } else {
            Arc::new(Mutex::new(MemorySet::clone(&self.memory_set.lock())))
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
            self.get_parent()
        } else {
            // 创建父子关系，此时以self作为父进程
            self.pid
        };
        let new_task = TaskInner::new(
            || {},
            String::new(),
            axconfig::TASK_STACK_SIZE,
            process_id,
            new_memory_set.lock().page_table_token(),
        );
        debug!("new task:{}", new_task.id().as_u64());
        TID2TASK
            .lock()
            .insert(new_task.id().as_u64(), Arc::clone(&new_task));
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
            if self
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
                if self
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
            self.tasks.lock().push(Arc::clone(&new_task));
            #[cfg(feature = "signal")]
            inner.signal_module.insert(
                new_task.id().as_u64(),
                SignalModule::init_signal(Some(new_handler)),
            );
            // inner
            //     .robust_list
            //     .insert(new_task.id().as_u64(), FutexRobustList::default());
            return_id = new_task.id().as_u64();
        } else {
            // 若创建的是进程，那么需要新建进程
            // 由于地址空间是复制的，所以堆底的地址也一定相同
            let new_process = Arc::new(Process::new(
                process_id,
                parent_id,
                new_memory_set,
                self.get_heap_bottom(),
            ));
            // 记录该进程，防止被回收
            PID2PC.lock().insert(process_id, Arc::clone(&new_process));
            new_process.tasks.lock().push(Arc::clone(&new_task));
            // 若是新建了进程，那么需要把进程的父子关系进行记录
            #[cfg(feature = "signal")]
            new_process.inner.lock().signal_module.insert(
                new_task.id().as_u64(),
                SignalModule::init_signal(Some(new_handler)),
            );

            // new_process
            // .inner
            // .lock()
            // .robust_list
            // .insert(new_task.id().as_u64(), FutexRobustList::default());
            return_id = new_process.pid;
            self.children.lock().push(new_process);
        };
        if !flags.contains(CloneFlags::CLONE_THREAD) {
            new_task.set_leader(true);
        }
        let current_task = current();
        // 复制原有的trap上下文
        let mut trap_frame = unsafe { *(current_task.get_first_trap_frame()) }.clone();
        drop(current_task);
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
}
