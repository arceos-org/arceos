# 第六周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

本周主要为学习代码框架，了解与确定实验内容。

### 分析 ArceOS 的 C app 运行方式

- `libax_bindings` 对 `rust_libax` 做了一层 C 接口的包装，并通过 `cbindgen` 生成头文件
- C app 的 include 目录为 `ulib/c_libax/include` 和 `ulib/c_libax/libax_bindings`
- 通过指定 features 来控制 `libax` 中使用的 modules，从而实现组件化
- 从 [musl.cc](https://musl.cc/) 下载交叉编译工具链，尝试编译运行了三个 C app

### 分析 smoltcp 的使用

- `smoltcp`：Rust 实现，分层提供 API

- `axnet` 通过 `smoltcp_impl` 对 `smoltcp` 进行包装，对下使用 `axdriver::NetDevices`，`driver_net` 等模块驱动网卡，对上提供 `TcpSocket` 以供应用使用 TCP
- 实验的一大目标即为实现 `lwip_impl` 来对 C 实现的 `lwip` 进行包装与适配

### 初识 LwIP

- 纯 C 实现，对内核中会使用到操作系统功能的地方进行了抽象，可以通过实现这些API完成迁移
- 支持协议较为完整
- 实现了一些常见的应用程序，完成迁移后或许都可以尝试跑起来

## 下周计划

- 进一步学习 `smoltcp_impl` 对底层驱动的使用方式
- 进一步学习 C 和 Rust 之间的适配方式
- ……

