# 第八周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

### 分析 LwIP 

#### netif 数据结构

- `LwIP` 对于网卡的抽象数据结构
- 多网卡组织为 `netif` 的链表，名为 `netif_list`
- 通过 `netif_add` 函数将网卡挂载到 `netif_list` 链表上，需要提供IP地址、子网掩码、默认网关、初始化网卡回调函数 `ethernetif_init`、收包回调函数等参数。在挂载前，`ethernetif_init` 会被调用。

#### 内存管理

- `LwIP` 可以自行管理内存。它的内存管理策略有两种：内存堆和内存池。

#### pbuf 数据结构

存储数据区域指针、长度、pbuf 类型、引用数量等信息的结构，可以组织成链表形式。

##### pbuf 类型

- `PBUF_RAM`：内存堆中分配，数据区域紧跟在 pbuf 结构体地址后（会预留 layer 的头部空间），协议栈中最常用
- `PBUF_POOL`：内存池中分配，数据区域紧跟在 pbuf 结构体地址后（会预留 layer 的头部空间），收包时用
- `PBUF_ROM`：内存池中分配，分配时不包含数据区域，数据区位于 ROM
- `PBUF_RAM`：内存池中分配，分配时不包含数据区域，数据区位于 RAM

### 移植 LwIP

先主要完成驱动的适配，以类似裸机的形式运行。

#### 需要实现的函数

- `low_level_init()`：网卡初始化函数，供 `ethernetif_init` 调用
- `low_level_output()`：网卡的发送函数，将 pbuf 数据结构解包并发出
- `low_level_input()`：网卡的接收函数，将收到的数据封装为 pbuf 数据结构
- `sys_now()`：时钟

#### 需要移植的头文件

- `lwipopts.h`
- `cc.h`
- `sys_arch.h`

### 移植 LwIP

在 `crate` 中创建模块 `lwip_rust`，目录组织如下：

- `custom/`：移植需要的文件
- `depend/`：以 git submodule 的形式导入的 lwip 库
- `src/`：包装为 rust 模块
- `build.rs`：编译和生成接口脚本，参考 https://github.com/eycorsican/leaf/blob/b0779107921683204a65bb1d41edc07a52688613/leaf/src/proxy/tun/netstack/bindings.rs
- `wrapper.h`：所有需要生成接口的头文件

## 下周计划

- 
