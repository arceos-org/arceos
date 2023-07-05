# Week 9 交流

> 郑友捷

## 本周工作内容

* 实现了`arceos`内核的多地址空间
* 实现基础的进程支持，支持进程的创建、回收并在此基础上实现了`exec`系统调用
* 与闭浩扬助教进行交流，了解`Maturin`项目的一些特性
* 讨论分工情况



## Arceos地址空间支持：分离编译

* 实现文件与用户库在脱离内核库下的单独编译：参考`rcore`编译`user`的方法
* 实现基础的文件加载与ELF文件读取：
  * 文件加载当前仅通过汇编直接写死，类似于`rcore`的`link_app.S`。后续会使用文件系统进行替代。
  * ELF文件读取：仅支持静态编译，与`rcore`类似，未考虑动态链接等情况（`Maturin`实现）。



## Arceos地址空间支持：页表支持

引入地址空间数据结构：

```rust
pub struct MemorySet {
    pub page_table: PageTable,
    pub areas: Vec<MapArea>,
}

pub struct MapArea {
    /// global page本身就是多个页面的，且存储了起始地址
    /// 存储了GlobalPage防止其因为生命周期结束而自动释放对应物理页面
    pub pages: GlobalPage,
    pub flags: MappingFlags,
}
```



## Arceos地址空间支持：单页表

* 单页表含义：从用户态trap到内核再返回到用户态的这个过程，若没有发生进程切换，则保持地址空间不变，即不切换`satp`。
* 单页表的优缺点：
  * 优点：提高性能，减少更换地址空间导致的换页性能消耗
  * 缺点：安全性下降。
  * 比赛中大多选择单页表保证性能。
* 内核初始化一个页表，之后作为只读页表，不进行修改。

* 为了保证内核在使用用户地址空间时也可以访问内核自身的代码，因此需要把内核的页表项复制到用户的页表中。

  ```rust
  /// 复制内核页表到用户页表
  pub fn copy_from_kernel_memory() -> PageTable {
      let page_table = PageTable::try_new().unwrap();
      let idx_len = PAGE_SIZE_4K / ENTRY_COUNT;
      for idx in 256usize..512 {
          // 由内核初始的虚拟地址决定复制多少大页
          let kernel_pte_address: *const usize = (phys_to_virt(KERNEL_PAGE_TABLE.root_paddr())
              .as_usize()
              + idx_len * idx) as *const usize;
          let kernel_pte = unsafe { core::slice::from_raw_parts(kernel_pte_address, idx_len) };
          let user_pte_address =
              (phys_to_virt(page_table.root_paddr()).as_usize() + idx_len * idx) as *mut usize;
          let user_pte = unsafe { core::slice::from_raw_parts_mut(user_pte_address, idx_len) };
          user_pte.copy_from_slice(&kernel_pte);
      }
      page_table
  }
  ```

* 在读取应用程序的ELF文件的时候，会进行用户地址空间的页表初始化，可以借鉴rcore的框架，复用arceos的接口进行地址空间的初始化。
* 地址空间仅在任务切换时会进行检查，若切换到的任务属于不同的进程，则会进行地址空间的切换。



## Arceos进程支持：数据结构

* `linux`内核与`Maturin`均将进程控制块与线程控制块进行统一，用一个数据结构进行管理。

* `unikraft`（贾越凯助教推荐的微内核）与`rcore`将进程控制块与线程控制块分开管理。

* 贾越凯助教建议不在`axtask`直接支持进程控制，防止兼容性不好。

* 最终数据结构如下：

  ```rust
  /// 进程控制块
  pub struct Process {
      /// 进程的pid和初始化的线程的tid是一样的
      pub pid: u64,
      /// 内部可变块
      pub inner: SpinNoIrq<ProcessInner>,
  }
  
  pub struct ProcessInner {
      /// 父进程
      pub parent: Option<Arc<Process>>,
      /// 子进程
      pub children: Vec<Arc<Process>>,
      /// 线程列表
      pub tasks: Vec<AxTaskRef>,
      /// 地址空间
      pub memory_set: MemorySet,
      /// 进程状态
      pub is_zombie: bool,
      /// 退出状态码
      pub exit_code: i32,
  }
  
  /// 线程内部可变块
  pub struct TaskInner {
      id: TaskId,
      name: &'static str,
      is_idle: bool,
      is_init: bool,
      /// 所属进程
      pub process: Arc<Process>,
      /// 是否是所属进程下的主线程
      is_leader: AtomicBool,
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
  }
  ```

  当前还未添加信号、文件描述符等结构，后续会添加。



## Arceos进程支持：新建一个进程

1. 新建一个地址空间，其页表复制内核页表的内容。
2. 读取ELF文件，并且将内容写入到地址空间中。
3. 根据地址空间新建一个进程，分配一个ID。
4. 新建一个线程`task`作为该进程的主线程，并且标记其父子关系。
5. 初始化`task`的`trap`上下文，将其写到对应的内核栈上。



## Arceos进程回收

* 回收物理页帧页表
* 将所有子进程转交给调度进程。
* 标记退出状态、返回值等等。



## Arceos进程：exec调用

相比于新建进程的区别：

1. 释放原先所有线程占用的用户态物理资源：直接释放除去内核态外的所有页面。
2. 读取ELF文件，写入新的程序到原来的地址空间中
3. 将运行参数写入到对应的用户栈上。
4. 将新的trap上下文写入到内核栈上。



## exec调用当前的问题

* 问题：若尝试释放用户态占用的物理页面，但是保留内核页帧，此时在为新的任务分配物理页面的时候，会分配原先任务使用的物理页面，体现了复用资源的思想。但是在访问该物理页面的时候会出现`page fault`异常。

* 猜测：MMU转化出现了问题，但是在确保satp不变的情况下依旧有这个问题。
* 解决方法：先保持原先的地址空间不释放，对新的任务申请新的物理页面，之后再将原先的地址空间释放。此时会避免bug出现。但是旧的地址空间页面是否完全释放仍然不能确定。



## Maturin学习工作

* 周一时和闭浩扬学长关于内核加载地址以及`maturin`内核的实现特性进行了线下交流。
* 在编写进程支持工作时也对`maturin`进一步了解



### 内核中断与用户中断

* 关于内核中断：
  * `rcore`：若中断发生在`__alltraps`，则此时未改变`stvec`，会导致死循环。
  * `arceos`：在`trap_vector_base`进入时通过判断`sscratch`是否为0来判断是否为内核中断。但是这种只能适用于第一次进入应用前的中断判断。
  * `maturin`：将内核地址加载在高地址，发生中断时将`sp`的值转化为带符号数。若大于0说明指向了用户栈，是用户中断。若小于0说明指向内核栈，是内核中断。

* 内核中断发生时，`maturin`会在原先内核栈的基础上进行trap处理，即内核栈嵌套。



### 特色应用：gcc

* gcc关键：动态链接文件的加载
* 涉及到ELF文件的读取与动态加载



### linux内核动态加载ELF文件流程如下

1. 填充并且检查目标程序ELF头部

2. load_elf_phdrs加载目标程序的程序头表

   > 程序表头：
   >
   > ```c
   > typedef  struct {
   >  unit32_t  p_type;    #数据类型
   >  uint332_t  p_flags; #标志位
   >  uint64_t  p_offset; #在ELF文件中的偏移
   >  uint64_t  p_vaddr;  #虚拟地址
   >  uint64_t  p_paddr;  #物理地址
   >  uint64_t  p_fllesz;  #在硬盘上的大小
   >  uint64_t  p_memsz;  #在内存中大小
   >  uint64_t  p_align;  #内存对齐方式
   > } Elf64_Phdr;
   > ```
   >
   > 反映某一段segement的类型。

3. 如果需要动态链接, 则寻找和处理解释器段

   > GNU把对于动态链接ELF映像的支持作了分工：
   >
   > 把ELF映像的装入/启动入在Linux内核中；而把动态链接的实现放在用户空间（glibc），并为此提供一个称为”解释器”(ld-linux.so.2)的工具软件，而解释器的装入/启动也由内核负责。

4. 检查并读取解释器的程序表头

5. 装入目标程序的段segment

6. create_elf_tables填写目标文件的参数环境变量等必要信息

7. start_kernel宏准备进入新的程序入口

### maturin中

> 对应写法：`kernel/src/loaders/mod.rs::Elfloader::init_vm`函数。
>

1. 先寻找解释器段，若存在解释器段则读取解释器的路径将其添加到启动参数中。
2. 装入原有ELF文件的静态段
3. 检查是否需要重定位，若有则说明是动态编译的，若有则需要根据动态加载的`.rela.dyn`等段落提供的内容进行重定位，修改或者添加符号。
4. 设置用户栈等内存内容。
5. 设置文件运行的环境变量，将其写入到用户栈上，此时用户栈上带有了运行参数。
6. 将用户栈写入地址空间。完成加载。

上述只是简述，其中有很多细节当前仍然未完全弄清，只能粗浅介绍，请原谅。



## 关于合作的想法

之前分工进行的较为艰难，主要是三人均负责进程部分，但都未实现地址空间等更为基础的东西，导致大家都得从基础开始做。

当前的进程支持仍然会有许多bug，不过基础的框架感觉搭起来了。依据初赛内容也许可以分为三个部分：

1. 进程管理：clone \ wait
2. 内存管理：包括堆的使用和文件加载等等
3. 文件管理：支持文件系统

按照这三个部分进行分工如何？



## 下周安排

1. 确定分工，实现clone等系统调用
2. 继续学习`maturin`的相关特性





## 老师建议

1. 对于`syscall`的参数传递、特权级切换等内容，写成一个脚本，对于所有系统调用都是通用的，如通用数据类型的加载以及指针数据的传递。

2. 可以参考linux等系统对于进程线程的定义、数据结构定义与实现，一定要做linux的常见集。

3. 了解其他组的工作，如做fs文件系统、调度算法、socket等内容，做的足够好的话可以拿来直接用。

4. 可以参考到qemu进行更为深层的调试
5. 考虑lazy分配内存、动态加载文件
6. 学习其他一等奖项目、Linux系统的整体实现