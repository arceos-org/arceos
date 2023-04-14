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

### 移植 LwIP 实操

#### 引入 LwIP 模块

在 `crate` 中创建模块 `lwip_rust`，目录组织如下：

- `custom/`：移植需要的文件
- `depend/`：以 git submodule 的形式导入的 lwip 库
- `src/`：包装为 rust 模块
- `build.rs`：编译和生成接口脚本，参考 https://github.com/eycorsican/leaf/blob/b0779107921683204a65bb1d41edc07a52688613/leaf/src/proxy/tun/netstack/bindings.rs
- `wrapper.h`：所有需要生成接口的头文件

#### 编译与链接

需手动在 `build.rs` 中指定 `base_config.flag("-march=rv64gc").flag("-mabi=lp64d").flag("-mcmodel=medany");`

否则会导致  `cannot link object files with different floating-point ABI from /tmp/rustcjJ6QUD/symbols.o` 

#### 踩坑

调试：`make A=apps/net/lwip_test/ ARCH=riscv64 LOG=trace NET=y MODE=debug debug`

##### 链接脚本问题

初始化 lwip 时出现 `Unhandled trap Exception(StorePageFault) @ 0xffffffc080206ce2`，gdb 跟踪调试发现访问了未在页表中的内存，进一步发现是在 lwip 访问 static 变量时出现。objdump 发现 .bss 段并未全部被包含在对应页表项中，于是怀疑 `axhal/linker.lds.S` 链接脚本中的 ebss 计算有问题。

从

```
.bss : ALIGN(4K) {
    boot_stack = .;
    *(.bss.stack)
    . = ALIGN(4K);
    boot_stack_top = .;

    sbss = .;
    *(.bss .bss.*)
    *(.sbss .sbss.*)
    . = ALIGN(4K);
    ebss = .;
}
```

改为

```
.bss : ALIGN(4K) {
    boot_stack = .;
    *(.bss.stack)
    . = ALIGN(4K);
    boot_stack_top = .;

    sbss = .;
    *(.bss .bss.*)
    *(.sbss .sbss.*)
}
. = ALIGN(4K);
ebss = .;
```

即可。

## 下周计划

- 
