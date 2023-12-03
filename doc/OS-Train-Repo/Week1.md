# Week1

> Author：郑友捷

本周主要工作为确定选题。

## 选题简介

StarryOS 的可插拔 syscall 模块实现

## 选题说明

当前的 StarryOS 是基于 Unikernel 架构的 ArceOS 并进行一定改造得到的宏内核。理论上它可以在编译期通过调整编译参数实现宏内核和 Unikernel 两种架构的选择性启动。但是由于比赛时期紧张，同时比赛的测例运行需求比较特殊，导致 StarryOS 现在对 Unikernel 架构的启动支持不是很好。

另外，由于比赛需求，StarryOS 启动宏内核时需要将即将运行的用户程序通过 riscv-gcc 编译为可执行二进制文件加入到文件镜像之后才能被运行，但运行 Unikernel 时只需要把用户程序的高级代码源码加载进来一同编译即可。为了更好地保持兼容性，我们希望把两种架构的启动入口都做成通过高级代码源码启动。



## 选题即将做的工作

1. 类比于rCore，添加一个用户库，将用户程序的函数调用转化为`ecall`汇编调用。
2. 改造编译参数，通过条件编译实现不同架构启动
3. 构造测例，测试不同架构下的程序性能：
   1. 负载测试性能
   2. 安全测试
4. 其他。。。



# 本周工作

## 在 Starry 代码基础上运行原有的 ArceOS 应用

原本 Starry 代码是基于 ArceOS 开发的，也始终保持着和 ArceOS 上游的同步。但是由于比赛时期较赶，且缺少规范的 CI-CD，并没有保证 Starry 代码可以始终通过原有 ArceOS 测例。但是现在为了做到两种架构的实时兼容，需要先保证能够通过各自的测例。



* 尝试通过 ArceOS 原有的 Hello World 测例

在尝试通过 Hello World 测例时，由于之前开发的时候对 Starry 更改的绝大多数内容都使用了条件编译，因此并不需要修改过多内容。

1. 首先尝试直接编译运行，看看能不能跑。。

   

   发现报错：

   ````shell
   error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait
   ````

   

   根据报错，应该是少了全局分配器。但是运行 ArceOS 原有代码时，发现并不需要实现全局分配器也可以编译出这个测例，所以应该是 Starry 在某一个地方引入了一个需要用到堆分配内存的代码，但没有做好兼容。

   

2. 对比 ArceOS 代码检查了一段时间，找不到问题，感觉文件量太大不好对比。

   

3. 后面思考：`global allocator`是用到堆内存分配机制，在不引入特殊语句的情况下，我们一般只会通过`extern crate alloc`引入`alloc`和与它相关的数据结构，才能做到访问、分配堆内存。

   因此尝试全局搜索所有引入了`alloc`的模块，逐个注释掉这些模块对`alloc`的直接引用，进行排查。

   仍然报错。。

   

4. 但是这个思路应该是对的，于是我根据所有引入了 `alloc` 的模块，看看编译时他们是否被引入，从而确定`alloc`是否被间接引用了。

   

   最后发现问题出在`axfs_ramfs`上，它实现了 ramfs 相关的文件系统信息。在 HelloWorld 测例中它不应该被引入，但是编译时发现它出现在了被编译清单上。

   <img src="../figures/train-week1-1.png" alt="avatar" style="zoom:50%;" />
   
   
   
   继续检查，发现`axhal`模块引用了这个 crate，最后查明是 trap 处理时使用到了这个 crate 下的一个文件。
   
   <img src="../figures/train-week1-2.png" alt="avatar" style="zoom:50%;" />
   
   这个文件是为了统计中断发生的次数，是当时决赛临时添加的一个文件，没有做好兼容导致出现了问题。
   
   将这个文件加上条件编译，同时设置`axfs_ramfs`为可选引入模块，从而解决了问题。

* task/yield 测例

  需要对 axtask 中一系列结构体成员与其对应的方法加上对应的条件编译语句。为了简约起见，为`TaskInner`新加了一个`impl`，集中存放为宏内核实现加上的内容。

* net/bwbench 测例

  没有串口输出，需要打开 `LOG=info` 开关才可以看到对应输出，部分输出截取如下：

  ```shell
  [  1.066117 0 axnet::smoltcp_impl::bench:35] Transmit: 0.773GBytes, Bandwidth: 6.184Gbits/sec.
  [  2.065980 0 axnet::smoltcp_impl::bench:35] Transmit: 0.840GBytes, Bandwidth: 6.720Gbits/sec.
  [  3.065980 0 axnet::smoltcp_impl::bench:35] Transmit: 0.744GBytes, Bandwidth: 5.956Gbits/sec.
  [  4.066010 0 axnet::smoltcp_impl::bench:35] Transmit: 0.768GBytes, Bandwidth: 6.150Gbits/sec.
  [  5.066014 0 axnet::smoltcp_impl::bench:35] Transmit: 0.745GBytes, Bandwidth: 5.962Gbits/sec.
  [  6.066012 0 axnet::smoltcp_impl::bench:35] Transmit: 0.742GBytes, Bandwidth: 5.937Gbits/sec.
  [  7.066013 0 axnet::smoltcp_impl::bench:35] Transmit: 0.741GBytes, Bandwidth: 5.930Gbits/sec.
  [  8.066012 0 axnet::smoltcp_impl::bench:35] Transmit: 0.747GBytes, Bandwidth: 5.978Gbits/sec.
  [  9.066032 0 axnet::smoltcp_impl::bench:35] Transmit: 0.744GBytes, Bandwidth: 5.955Gbits/sec.
  ```

  



