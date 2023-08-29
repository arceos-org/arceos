# Week 12

## Maturin运行决赛测例指令

### 生成对应文件镜像并运行

目前可以加载 `libc` 测例或 `busybox/lua/lmbench` 测例，默认为 `libc`。

生成文件测例方式：

```shell
cd kernel
make clean
DISK_DIR=busybox make testcases-img
```

或直接在`/kernel/Makefile` 里第 12 行直接修改 `DISK_DIR ?= libc` 一项。

当前应当可以执行两类测例：`libc`与`busybox`。

之后直接执行`make run `即可。

### 代码中执行流程：

1. `kernel/src/file/device/test.rs`中`lazy_static`部分，`TESTCASES_ITER`指明了当前运行的测例，`TEST_STATUS`指明了所有即将运行的测例，

2. `kernel/src/task/scheduler.rs`中`lazy_static`部分。`IS_TEST_ENV`指明了是在测试环境下。此时会通过执行`load_next_testcase`来读取测例。

3. `load_next_testcase`会读取一条指令并利用该指令执行对应的程序。当前的默认`TESTCASES`为

   ```rust
   "busybox sh lua_testcode.sh",     // lua 测例
   "busybox sh busybox_testcode.sh", // busybox 测例
   "busybox sh lmbench_testcode.sh", // lmbench 测例
   ```

4. 在上述例子中，会启动`busybox`应用程序，并将`sh`以及后续的文件名传递给该应用程序。在`busybox`支持下会执行`sh`指令，进而去执行一系列的测试指令。

### 测例说明

#### busybox

##### busy说明

BusyBox 是一个开源项目，它提供了大约 400 个常见 UNIX/Linux 命令的精简实现。

##### busybox测试过程

* 通过上述`testcases`执行`busybox_testcode.sh`

* 真正执行的指令在`busybox_cmd.txt`中，包括了很多条指令。这些指令是linux对应指令的子集，所需要支持的操作也是linux所需要操作的子集。包括如cat，echo等常见指令。

  ```sh
  echo "#### independent command test"
  ash -c exit
  sh -c exit
  basename /aaa/bbb
  cal
  clear
  date 
  df 
  dirname /aaa/bbb
  dmesg 
  du
  expr 1 + 1
  false
  true
  which ls
  uname
  uptime
  printf "abc\n"
  ps
  pwd
  free
  hwclock
  kill 10
  ls
  sleep 1
  echo "#### file opration test"
  touch test.txt
  echo "hello world" > test.txt
  cat test.txt
  cut -c 3 test.txt
  od test.txt
  head test.txt
  tail test.txt 
  hexdump -C test.txt 
  md5sum test.txt
  echo "ccccccc" >> test.txt
  echo "bbbbbbb" >> test.txt
  echo "aaaaaaa" >> test.txt
  echo "2222222" >> test.txt
  echo "1111111" >> test.txt
  echo "bbbbbbb" >> test.txt
  sort test.txt | ./busybox uniq
  stat test.txt
  strings test.txt 
  wc test.txt
  [ -f test.txt ]
  more test.txt
  rm test.txt
  mkdir test_dir
  mv test_dir test
  rmdir test
  grep hello busybox_cmd.txt
  cp busybox_cmd.txt busybox_cmd.bak
  rm busybox_cmd.bak
  find -name "busybox_cmd.txt"
  ```

* 对每一条指令都会执行相关操作并且收集结果。



#### lmbench

##### lmbench说明

lmbench是一个性能测试工具，可以进行相关的性能测试。

##### lmbench测试过程

* 通过上述`testcases`执行`lmbench_testcode.sh`

* 测例内容：

  ```sh
  #!/bin/bash
  echo latency measurements
  lmbench_all lat_syscall -P 1 null
  lmbench_all lat_syscall -P 1 read
  lmbench_all lat_syscall -P 1 write
  busybox mkdir -p /var/tmp
  busybox touch /var/tmp/lmbench
  lmbench_all lat_syscall -P 1 stat /var/tmp/lmbench
  lmbench_all lat_syscall -P 1 fstat /var/tmp/lmbench
  lmbench_all lat_syscall -P 1 open /var/tmp/lmbench
  lmbench_all lat_select -n 100 -P 1 file
  lmbench_all lat_sig -P 1 install
  lmbench_all lat_sig -P 1 catch
  lmbench_all lat_sig -P 1 prot lat_sig
  lmbench_all lat_pipe -P 1
  lmbench_all lat_proc -P 1 fork
  lmbench_all lat_proc -P 1 exec
  busybox cp hello /tmp
  lmbench_all lat_proc -P 1 shell
  lmbench_all lmdd label="File /var/tmp/XXX write bandwidth:" of=/var/tmp/XXX move=1m fsync=1 print=3
  lmbench_all lat_pagefault -P 1 /var/tmp/XXX
  lmbench_all lat_mmap -P 1 512k /var/tmp/XXX
  busybox echo file system latency
  lmbench_all lat_fs /var/tmp
  busybox echo Bandwidth measurements
  lmbench_all bw_pipe -P 1
  lmbench_all bw_file_rd -P 1 512k io_only /var/tmp/XXX
  lmbench_all bw_file_rd -P 1 512k open2close /var/tmp/XXX
  lmbench_all bw_mmap_rd -P 1 512k mmap_only /var/tmp/XXX
  lmbench_all bw_mmap_rd -P 1 512k open2close /var/tmp/XXX
  busybox echo context switch overhead
  lmbench_all lat_ctx -P 1 -s 32 2 4 8 16 24 32 64 96
  ```

* 相关提及的指令说明：

  * lmbench_all：运行所有的lmbench测例数据，即执行一条lmbench_all指令

  * lat_syscall指令：用于测量系统调用的延迟时间。可以指定系统调用执行的次数来提高测试精确度。

    > lat_syscall测试指令是针对系统调用延迟时间的基准测试，它并不涉及系统调用的吞吐量或其它方面的性能评估。如果需要进行系统调用的吞吐量测试，可以使用lmbench测试工具中的其它测试指令，如lat_pipe或lat_tcp等。

  * lat_select：用于测量select系统调用的延迟时间。会在指定的进程数中启动一个循环，每次循环会调用select系统调用，并测量该系统调用的延迟时间，以评估系统的性能和稳定性。

    > select是一种I/O多路复用机制，用于在多个文件描述符上进行等待，直到其中一个或多个文件描述符变为可读、可写或发生异常事件时返回。select系统调用通常用于实现异步I/O或同时处理多个I/O事件的应用程序。
    >
    > 
    >
    > 测试select需要一个文件，测例中指定了file作为文件。如果未指定`file`参数，则`lat_select`指令会在内存中创建一个虚拟的文件来进行测试。
    >
    > 

  * lat_sig：测试信号的处理延迟时间。该指令可以通过模拟发送信号和信号处理函数的执行来测量系统的响应时间。它将会模拟发送信号并等待信号处理函数执行完成，并测量整个过程的延迟时间。

  * lat_pipe：用于测试管道（pipe）的性能和延迟时间。它将会创建一个管道，然后通过多个进程进行读写操作，测量数据传输和同步的延迟时间。

  * lat_proc：用于测试进程创建的性能和延迟时间。

  * lmdd：测试磁盘的读写性能和延迟时间。它将会打开指定的测试文件，然后通过多个线程进行读写操作，测量磁盘的读写性能和延迟时间。它可以测试顺序读写、随机读写等多种模式，可以指定块大小、IO深度等测试参数。

  * bw_pipe：用于测试进程间管道通信的**带宽**和延迟时间。

  * bw_file_rd：测试文件读取操作的**带宽性能**。该指令通过多个进程并行读取指定文件，并计算平均吞吐量和标准差等统计信息。

  * bw_mmap_rd：测试内存映射文件的读取**带宽性能**。该指令通过在多个进程中并行读取指定的内存映射文件，并计算平均吞吐量和标准差等统计信息。

  * lat_ctx：测试上下文切换的性能。在测试过程中，该指令会创建多个线程，并在不同的线程之间进行上下文切换，从而计算上下文切换的延迟时间和吞吐量等指标。

  * lat_pagefault：测试页面故障（page fault）的性能。在测试过程中，该指令会模拟页面故障，并计算产生页面故障时的延迟时间和吞吐量等指标。

  * lat_fs：测试文件系统的性能。该指令通过对指定文件的读写操作，来测试文件系统的延迟和吞吐量等性能指标。

  * lat_mmap：测试内存映射文件的性能。该指令通过对内存映射文件的读写操作，来测试系统的延迟和吞吐量等性能指标。

    * 与`bw_mmap_rd`区别：

      `lat_mmap`主要用于测试内存映射文件的延迟性能，即测试对内存映射文件进行读写操作所需要的时间。该指令通过对内存映射文件的读写操作，来测试系统的延迟和吞吐量等性能指标。

      `bw_mmap_rd`则主要用于测试内存映射文件的带宽性能，即测试对内存映射文件进行连续读取所能达到的最大带宽。该指令通过对内存映射文件进行连续的读取操作，来测试系统的带宽性能指标。

#### lua

##### lua说明

Lua是一种轻量级、高效、可嵌入的脚本编程语言。通过其核心代码上提供的交互式程序可以方便的进行编程，交互模式类似于python。

##### 测试方式

* lua的测试应当要求OS内核能够正确运行lua核心代码的程序`testcases/busybox/lua`，并在此基础上支持lua脚本应用。

* 通过上述`testcases`执行`lua_testcode.sh`，并且运行不同的`lua`脚本程序。



#### libc测例

##### libc测例说明

libc测例与busybox类测例不同，不需要依靠busybox执行，而是类似于初赛阶段的测例，给出一段代码之后将其在内核上直接运行。其分为三类：

* 初赛测例部分。
* libc静态测例：该部分不需要动态加载。
* libc动态测例：该部分需要动态加载，因此需要我们改造读取elf文件的函数，将其支持动态加载。

##### 运行方式

在生成文件镜像时指定`DISK_DIR`为`libc`即可。



## aeroOS研究

### OS特性

* 64位高半内核：即将应用程序的地址分配在低半部分，将内核代码分配在高半部分的虚拟地址。如将范围留`0x00000000 - 0xBFFFFFFF`用于用户代码，数据，堆栈，库等。具有这种设计的内核被称为“上半部分”。

  > 优点：方便链接与分配内存

* 4级或5级分页

* 抢占式percpu机制调度

  * 抢占式：进程可以在执行时被打断，或者被其他CPU接过控制权
  * percpu：为了避免多个 CPU 对全局数据的竞争而导致的性能损失，percpu 直接为每个 CPU 生成一份独有的数据备份，每个数据备份占用独立的内存，CPU 不应该修改不属于自己的这部分数据，这样就避免了多 CPU 对全局数据的竞争问题。

* Modern UEFI bootloader

* Symmetric Multiprocessing：拥有超过一个以上的处理器，这些处理器都连接到同一个共享的主记忆体上，并由单一作业系统来控制。

* On-demand paging：需要时分配页面，即类似于懒分配。

### 启动方式

1. 安装相应依赖

   在管理员模式下运行下列指令安装相应的依赖

   ```sh
   sudo ./tools/deps.sh
   ```

2. 运行内核

   内核启动的详细指令存储在`./aero.py`中，包含了一系列编译选项来选择不同的启动模式。

   * 直接运行指令

     ```sh
     ./aero.py
     ```

   * 会报错` No such file or directory: 'qemu-system-x86_64'`。查阅`./aero.py`发现其不支持`riscv`指令集，仅支持`x86和aarch`。

   * 尝试安装`qemu-system-x86_64`：在`rCore-tutorial`的环境配置的基础上，在`qemu-7.0.0`的文件夹下执行指令：

     ```sh
     ./configure --target-list=x86_64-softmmu,x86_64-linux-user
     make -j$(nproc)
     ```

     此时执行指令

     ```sh
     qemu-system-x86_64 --version
     ```

     会输出相关的版本信息，说明安装成功。

   * 再次尝试运行`./aero.py`，再次报错：`Could not initialize SDL(x11 not available) - exiting`，查询得知是因为`qemu`未安装图形化界面。

   * 查询指令，再次执行qemu安装，运行下列指令：

     ```sh
     ./configure --target-list=x86_64-softmmu,x86_64-linux-user --enable-sdl
     make -j$(nproc)
     ```

   * 仍然报错，发现需要Ubuntu系统支持图形化界面。可以采用xcvrv客户端的形式。因此需要进行一系列配置。
   
   * 发现问题：若是直接运行给定的`./aero.py`，则无法找到程序入口，因为缺少了`host-rust`依赖安装。若是尝试构建完整的`./aero.py --sysroot`，则会因为众多依赖问题无法完成安装。

## 初赛部分

* 通过了除去文件管理外的所有测例
* 实现了自动化测试并且输出测试结果



## 下周工作

1. 合并文件系统
2. 尝试解决图形化界面OS的配置
3. 初步开始构建决赛的实现
