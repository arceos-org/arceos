# Week 2

> Author：郑友捷

## 和向老师讨论

第一次交流想要做一个宏内核和 Unikernel 的兼容，计划通过改变编译选项，让相同的源代码能够在不同的架构下运行，从而方便我们在开发应用代码时引入特权级等调试方式。但在第二次交流的时候，老师提到想让我做一个定制化的宏内核，也就是说可以根据需要去选择根据不同的模块来启动宏内核。

我理解这应该是一个进阶的过程，第一次交流是编译时调整架构启动，第二次是对宏内核进行尝试类似于 Unikernel 拆分模块的功能。

我想之后我的计划是：

1. 先做到第一步，并设计一些测例来测试 Unikernel 启动下和宏内核启动下的区别。
2. 若有时间，再尝试第二步，即试着拆分出一些模块，估计是从小往大拆，这一步之前没有尝试过，所以不确定难度多大。



## 本周工作

将宏内核的相关内容和 Unikernel 工作尽可能进行分开，从而尽可能实现两者对共同底层模块的复用与区分。



### 通过 feature 划分

当前想要做到的是编译期进行架构的区别，因此我选择通过 feature 的方式对两者进行区分。



由于宏内核默认是启动所有的模块，从而构成一个整体（讨论的定制化宏内核除外），因此我们设置一个 feature 为 monolithic，代表启动宏内核架构。当这个 feature 启动时，其他所有的模块都会被包含进来，如文件系统、信号模块、进程控制等等。



### 模块划分

为了更好地进行模块复用，我们先需要对 Starry 中的模块关系进行划分，将其根据 **宏内核是否需要** 和 **ArceOS 是否拥有** 两个标准进行划分。

![image-20231007202335681](../figures/train-week2-1.png)

具体对应到 Starry 上拥有的模块关系如下：

* ArceOS 中可直接沿用：log、driver 以及一系列解耦的 crate
* ArceOS 中需要适配：任务模块 axtask 、trap异常处理 axhal 等
* ArceOS 中需要添加：地址空间、进程、信号、文件系统、用户库



接下来就上面三种情况提到的不同模块进行详细讨论：

#### 可以直接沿用的模块

对于 ArceOS 中可以直接沿用的内容，我们不需要对其进行更改，可以直接复用。

但需要注意的是，为了实现宏内核，需要在 ArceOS 原有模块或者 crate 的基础上添加一些新的功能。

具体对应的内容有：

* crate：大部分可以复用，但是对于 page table 需要添加一些函数方便查询与插入。
* axlog：可以直接复用
* axdriver： 为了更加快速地读取比赛测例，需要添加 ramdisk 支持，将外部磁盘的内容拷贝到 ramdisk 上。



#### 需要适配的模块

这部分提到的模块需要做出较大部分的修改。相比于上一部分提到的模块，这里的模块需要在不同架构下进行不同 的处理，所以需要引入 feature 进行分支处理。

**值得强调的是，当前宏内核的适配部分仅适配了 riscv 架构，并未适配其他架构。**

##### axhal

在宏内核情况下，需要对更多的 trap 进行额外的处理，如用户系统调用异常、 page fault 异常等内容。具体修改内容在`axhal/src/arch/riscv/trap.rs`部分。部分修改如下：

```rust
match scause.cause() {
    Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
    Trap::Interrupt(_) => crate::trap::handle_irq_extern(scause.bits()),
    #[cfg(feature = "monolithic")]
    Trap::Exception(E::UserEnvCall) => {
        enable_irqs();
        // jump to next instruction anyway
        tf.sepc += 4;
        // get system call return value
        let result = handle_syscall(
            tf.regs.a7,
            [
                tf.regs.a0, tf.regs.a1, tf.regs.a2, tf.regs.a3, tf.regs.a4, tf.regs.a5,
            ],
        );
        // cx is changed during sys_exec, so we have to call it again
        tf.regs.a0 = result as usize;
    }
    #[cfg(feature = "monolithic")]
    Trap::Exception(E::InstructionPageFault) => {
		...
    }

    #[cfg(feature = "monolithic")]
    Trap::Exception(E::LoadPageFault) => {
        ...
    }

    #[cfg(feature = "monolithic")]
    Trap::Exception(E::StorePageFault) => {
        ...
    }

    _ => {
        panic!(
            "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
            scause.cause(),
            tf.sepc,
            tf
        );
    }
}
```

##### axtask

任务模块需要修改的内容较多。在宏内核下，原先 ArceOS 的任务模块承担了宏内核中的线程方面的功能。而为了保证**尽量不对 ArceOS 原有模块过多修改**，方便 ArceOS 复用，因此我们将 进程 和 线程控制块分开了。

在这种设计下，线程需要在原有基础上记录更多内容，比如所属进程 ID、计时器信息、CPU亲和力信息（用于实现`SCHED_SETAFFINITY`等系统调用）等，用于完成一系列的 Linux 系统调用。

我考虑了两种设计方法：

1. 在原有任务结构的基础上通过 feature 的方式添加成员域
2. 新建一个任务结构，继承原有的任务结构，以 trait 的方式实现宏内核相关的内容，在不同架构下选用不同的结构

为了实现方便起见，我选用了第一种。

因此我对任务结构的修改如下：

```rust
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

    #[cfg(feature = "monolithic")]
    pub sched_status: UnsafeCell<SchedStatus>,

    #[cfg(feature = "monolithic")]
    /// 退出时是否向父进程发送SIG_CHILD
    pub send_sigchld_when_exit: bool,
}
```

可以看到其上添加了许多 feature 信息，看上去虽然有些冗余，但也可以勉强完成两者区分的目的，同时不会过度影响上层模块对于 TaskInner 结构的使用。



另外，除去对结构体成员域的控制，还需要考虑函数运行语句的分支控制问题。我们以启动语句为例。

* 对于 Unikernel 的启动，由于不涉及地址空间的改变，可以直接通过调用任务入口函数的方式来进入到任务的执行中。
* 对于宏内核的启动，需要涉及到特权级的切换，因此要进行一些额外的处理。

两者的启动逻辑并不相同，因此需要通过 feature 进行区分。

```rust
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
```



通过以上两个模块展示的内容，可以看出 feature 的使用基本有两种情况：

1. 通过 feature 添加成员域或者额外处理语句
2. 通过 feature 选择不同的分支语句



通过上述两种方法，可以在尽可能减小修改量的同时完成两种架构的运行方式区分。



#### 需要新增的模块

相比于 Unikernel，宏内核需要新增地址空间、进程管理等一系列模块，但是这与我们想要做的与 Unikernel 兼容的内容关系不大。因此不在这里过多阐述。



### Tip

本周工作内容较多，一周暂时做不完，上面仅提出的是工作的目标，还需要进一步调试。。。

