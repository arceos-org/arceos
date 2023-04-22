# 追溯用户态建立过程

> 以riscv环境为例

1. 初始时进入`text`汇编代码段，第一个执行`text.boot`段代码

2. `axhal/src/platform/qumu_virt_riscv/boot.rs`中的`__start`函数位于`boot`顶部，执行一系列初始化操作，此时会调用`platform_init`函数。

3. `axhal/src/platform/qemu_virt_riscv/mod.rs`中执行`platform_init`函数，其中第二条指令的`set_tap_vector_base`设置了`stvec`的地址。

4. `axhal/src/arch/riscv/mod.rs`中执行`set_tap_vector_base`函数，设置stvec为汇编函数`trap_vector_base`的地址。

5. 汇编函数`trap_vector_base`定义在`axhal/src/arch/riscv/trap.S`，会判断当前trap来自于S还是U。并且进入对应的`trap_handler`，当前两个中断的处理函数相同，之后会进行区分。  执行完`trap_handler`之后会执行`restore_regs`汇编函数，回到原先的状态。

   相比于`rcore`，此时的`trap_vector_base`是之前`all_trap`和`restore`的结合体。



# trap上下文理解

1. 第一次进入应用程序时，需要初步将trap上下文保存在用户对应的内核栈上，之后使用restore函数读取trap上下文对应用进行初步初始化。之后调用sret回到用户态。
2. 用户进入到trap之后，跳转到`set_tap_vector_base`，此时内核会将用户的trap上下文存储在对应的内核栈中。
3. 当内核处理trap时，如果需要读取应用的trap上下文，直接从应用对应的内核栈读取一定内存并且直接转化为`trapcontext`类型。 
4. 当返回应用态时，重新读取trap上下文即可。



# rcore应用如何读取trap上下文

应用的task context中的ra存储的是trap_return的地址，在完成了task_switch之后会执行ret操作直接跳转到这个地方。

在加入任务切换之后，内核栈存储的是task_context，而应用的trap上下文存储在应用自身的次高虚拟页面。对应每一个正在运行的任务，`sscratch`指向了对应的次高虚拟页面。

内核读取应用trap上下文：该次高虚拟页面在对应任务的地址空间的物理页面。

应用返回时读取trap上下文：从对应虚拟页面读取。



# rcore如何初始化程序

程序初始化写一个trap context在对应的次高虚拟页面，关键有两个

1. 对应的用户栈的地址
2. 对应的sepc，即进入程序之后开始执行的代码位置。

其他均可以设置为0，包括ra等。

然后调用sret，进入到应用代码中。

上述内容被分为两个部分做完，第一个是建立任务时的app_init_context，第二个是task_context存储的trap_return执行的__restore函数。



切换完任务的上下文之后，会切换到对应的ra，此时存储的是上一个任务上次切换时运行到的位置。一般切换之后后续没有操作，即trap handler执行完毕，此时会自动执行trap return。

对于新任务，switch 的时候task上下文的ra需要给一个初始值，因此直接规定为 trap return。



在没有实现任务切换前：

内核栈保留的是各自的trap context。因为这个时候是通过直接使用汇编代码__restore进行返回的，不用分配函数栈。

在实现了任务切换之后：

trap context保存在各自的次高虚拟页面。

内核栈这个时候是用来执行trap return所需要的栈，不一定存在特定内容。毕竟某一个函数执行的时候你得给它留一个栈空间。



# arceos初始化程序

区别：

1. 当前为多线程体系，并没有明确的进程划分，task是属于是线程而不是进程。
2. 没有__restore函数
3. task_context存储的入口在`axtask/src/task.rs/task_entry`函数，也是通过switch切换到这里。理论上讲这里每一个应用只会走一次。
4. 所以进程只有一个，我们只需要在最开始运行的时候加入restore函数即可。
5. 因为这时候没有任务切换的需求，没有多地址空间，所以内核栈可以直接拿来存储trap context。
6. `axruntime/lib.rs`的`main`是当前进程的开始入口，所以需要对他进行trap的初始化。即更新sp，ra，通用寄存器等内容，然后执行sret，即restore函数。

# 需要我们做的内容

在进入应用程序之前写一个trap上下文切换，通过sret切换到U态。



# 遇到的问题

1. 切换特权级之后无法访问内存

   1. boot.rs开启了MMU
   2. axruntime的paging特性改变了satp寄存器
   3. riscv.toml设置了虚拟地址用于存储页表地址

   问题：开启了页表之后，当切换到U态就无法访问原先S态页表的内容。因此无法在单页表情况下完成转移。

2.  对于arceos的trap切换机制不是很熟悉。花了接近一天时间比对两者trap关系。

# 接下来工作

1. 分离多个页表，引入虚拟地址
2. 分离多个进程
