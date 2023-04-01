
# 第六周报告

蒋昊迪

---

## 选择项目

+ ArceOS 的改进——微内核

## 主要工作

+ 熟悉环境
+ 调研工作

--- 

## MicroOS

+ 进一步对 OS 进行模块化
+ 将非核心模块（各类驱动，FS，网络栈等）移至用户态
+ 特点
  - 👍 模块化：更容易定制化
  - 👍 安全：减少内核态的代码量
  - 👍 稳定：外围驱动崩溃不影响内核运行
  - 👎 性能：带来更多上下文切换

---

## 已有工作——Redox

+ 使用 Rust 编写的微内核 OS
+ 实现了 GUI 和一系列实用程序
+ 实现了 libc 和 Rust std
+ https://www.redox-os.org/zh/

--- 

## URL

+ Redox 将 URL 引入内核，实现了一种新的 IPC 方式
+ URL: `[scheme]:[reference]`，其中 `scheme` 由各种外围模块注册，相当于”协议“。
  - 例如：`file:/path/to/file` 通过 FS 模块完成文件 I/O
+ 用户程序通过文件/socket 方式与各类外围模块进行通信，一个打开的 `scheme` 称为 `resources`
+ "Everything is a URL"
  - 不是 file 的原因是，Redox 认为 file 的抽象在某些特殊情况下在逻辑上不合理。

--- 

## ArceOS 框架熟悉

+ `Helloworld` 流程

---

## 编译链接

+ 每个应用程序会依赖 OS 的一部分功能和特性。根据 `Cargo.toml` 来指定。
+ 每个应用程序单独编译形成镜像
+ 链接脚本位于 `modules/axhal/`，值得注意的是，起始地址位于 `0xffffffc080200000`。

--- 

## 启动流程

+ QEMU 启动，SBI 初始化，进入 S 态 (`0x80200000`)
+ 完成平台相关初始化工作(`modules/axhal/src/platform/qemu_virt_riscv/boot.rs`)，包括内存初始化，中断异常初始化等。
  - 注意此时页表被写入 `0xffffffc08000000` 和 `0x0000000080000000` 两页。
  - 镜像被装载在低地址，在页表写入后的函数跳转均采用高地址，在页表进行翻译。
  - 只需要页表设置的 Assembly 是 position independent 的就可以保证运行正确性。
+ 完成 OS 初始化工作，在 hello world 中相对较少，主要为一些附加功能的初始化。
+ 进入 `main` 函数执行。
  - 由于程序依然运行在 S 态，因此所有的”系统调用“都是直接调用对应函数执行的。
  - 这里是 `println!`，实现和 rCore 类似。

---

## 下一步工作

+ 实现 U/S 态切换，可能包括
  - trap 切换
  - syscall 设计
  - 用户库实现
+ 基于 U/S 态的页表管理
+ IPC
+ 模块进程化

--

# 谢谢
