# Week14

## 本周进展

1. 通过了初赛所有测例
2. 完成了决赛部分测例的运行，同时实现了批量静态链接，即将实现动态链接。
3. 初步构造了信号结构。
4. 学习umi等操作系统



## 通过测例

初赛部分测例已经通过，但是存在部分尚未解决的问题：

1. arceos文件系统当前不支持对物理地址不连续的buff的直接修改，只能通过逐页写入buffer的形式来修改缓冲区。

   这种情况仅会出现在lazy alloc中，因为lazy alloc得到的物理地址是不连续的。

2. 部分引用了arceos文件系统的内容会导致奇怪的bug，如实现getdirent系统调用时若缓冲区开的太小，即使该部分没有被显式调用，仍然会导致其他测例运行错误。



## 决赛部分测例运行

1. 实现了批量的静态链接，已经可以开始运行。
2. 在此基础上加入了动态链接的部分，但未经过测试。



## umi等操作系统学习启示

### 杭电的操作系统lastWakeUp

#### 特点

1. 实现了bash界面。
2. 内存管理进展较快，实现了copy on write。
3. 实现了并发程序，支持多核运行。
4. 合作上：采取线下集体开发的形式进行合作，同时也积极请教学长

5. 实现了进程和进程组的概念，进程结构如下：

   ```c
   // Per-process state
   struct proc {
       struct spinlock lock;
   
       // p->lock must be held when using these:
       enum procstate state; // Process state
       void *chan;           // If non-zero, sleeping on chan
       int killed;           // If non-zero, have been killed
       struct list_head head_vma;
       int exit_state; // Exit status to be returned to parent's wait
       pid_t pid;      // Process ID
   
       // maybe also need thread lock to access, p->tlock must be held
       struct spinlock tlock;
   
       // these are private to the process, so p->lock need not be held.
       uint64 kstack;                // Virtual address of kernel stack
       uint64 sz;                    // Size of process memory (bytes)
       pagetable_t pagetable;        // User page table
       struct trapframe *trapframe;  // data page for trampoline.S
       struct context context;       // swtch() here to run process
       struct file *_ofile[NOFILE]; // Open files
       struct inode *_cwd;          // Current directory
       // struct file *ofile[NOFILE];  // Open files(only in xv6)
       // struct inode *cwd;           // Current directory(only in xv6)
       char name[16]; // Process name (debugging)
   
       // wait_lock must be held when using this:
       struct proc *parent; // Parent process
   
       struct list_head state_list;   // its state queue
       struct proc *first_child;      // its first child!!!!!!!
       struct list_head sibling_list; // its sibling
   
       int sigpending;                   // have signal?
       struct signal_struct *sig;        // signal
       sigset_t blocked;                 // the blocked signal
       struct sigpending pending;        // pending (private)
       struct sigpending shared_pending; // pending (shared)
   
       tgid_t tgid;                   // thread group id
       int thread_cnt;                // the count of threads
       struct list_head thread_group; // thread group
       struct proc *group_leader;     // its proc thread group leader
   
       pgrp_t pgid; // proc group id
   
       struct list_head wait_list; // waiting  queue
       pid_t *ctid;
       // struct spinlock wait_lock;
       struct semaphore sem_wait_chan_parent;
       struct semaphore sem_wait_chan_self;
   
       long tms_stime;   // system mode time(ticks)
       long tms_utime;   // user mode time(ticks)
       long create_time; // create time(ticks)
       long enter_time;  // enter kernel time(ticks)
   };
   
   ```

   

#### 调试特点

1. 使用vscode连接到debugger上，通过vscode上的gdb进行调试。
2. 使用trace追踪





# umi

## 进展

1. co-trap实现函数式trap分离

2. 实现了支持非异步函数和异步函数的函数调用表

3. 较好的泛型操作
4. 较好的异步实现操作
5. 无栈协程操作

## 特点

1. 较多的rust高阶操作：包括泛型、宏展开
2. 内核态中断
3. 大量的异步操作与生命周期指令
4. 进程部分将任务的状态作为局部变量传入函数，尽可能减少锁结构，减小锁的粒度。通过减少锁的使用使得调试更为方便。
5. 协程不是为了性能，而是为了调试方便
6. 通过feature的方式使得模块与内核解耦合，提供了一个trait供内核使用
7. ECS系统引入函数调用表，统一使用了map。通过请教本人，该写法可能是为了使得代码更加简洁，同时索引的方式可以减少匹配耗时。



**其开发过程中很多设计是为了更方便地进行调试使用地，是一个比较新颖的开发方式**。

**同时通过提供feature的方式使得模块解耦是一个值得借鉴的思想，我们的arceos的文件系统模块可以利用这一点。**



### 下周安排

1. 完成在线评测
2. 完成动态链接
3. 实现信号结构
4. 开始考虑并发多核启动问题