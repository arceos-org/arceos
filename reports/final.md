# 组件化OS--aceros的改进：支持和优化lwip网络协议栈

**致理-信计01  佟海轩  2020012709**

## 主要成果

- 将 lwip 协议栈接入 aceros，支持 **TCP / UDP / DNS**，支持 **IPv6**，提供与 smoltcp 相一致的网络接口，支持现有的各个网络应用
- 对 lwip 的适配进行分析与优化，最终**性能**与使用 smoltcp 相当，**稳定性**更高
- 与吴大帅同学的项目 [arceos-udp](https://github.com/reflyable/arceos-udp) 配合，对其中的 UDP，DNS 和网络应用提供基于 lwip 的支持



## 移植 lwip

### lwip 概述

#### netif 数据结构

- `lwip` 对于网卡的抽象数据结构
- 多网卡组织为 `netif` 的链表，名为 `netif_list`
- 通过 `netif_add` 函数将网卡挂载到 `netif_list` 链表上，需要提供IP地址、子网掩码、默认网关、初始化网卡回调函数 `myif_init`、收包回调函数等参数。在挂载前，`myif_init` 会被调用。

#### 内存管理

- `lwip` 可以自行管理内存。它的内存管理策略有两种：内存堆和内存池。
- 内存池：每个池用于分配固定大小的内存块，速度块
- 内存堆：分配不固定大小的内存

#### pbuf 数据结构

存储数据区域指针、长度、pbuf 类型、引用数量等信息的结构，可以组织成链表形式。

##### pbuf 类型

- `PBUF_RAM`：内存堆中分配，数据区域紧跟在 pbuf 结构体地址后（会预留 layer 的头部空间），协议栈中最常用
- `PBUF_POOL`：内存池中分配，数据区域紧跟在 pbuf 结构体地址后（会预留 layer 的头部空间），收包时用
- `PBUF_ROM`：内存池中分配，分配时不包含数据区域，数据区位于 ROM
- `PBUF_RAM`：内存池中分配，分配时不包含数据区域，数据区位于 RAM

### 与驱动适配

先完成驱动的适配，以类似裸机的形式运行。

#### 分析

##### 需要移植的头文件

- `lwipopts.h`：协议栈的各种参数，先将 `NO_SYS` 设为 1 以裸机形式运行
- `arch/cc.h`：编译器与体系结构相关的设置
- `arch/sys_arch.h`：用于适配系统的相关设置

##### 需要实现的函数

- `err_t myif_init(struct netif *netif)`：网卡初始化函数，作为 `netif_add` 的参数，在添加网卡时初始化
- `err_t myif_link_output(struct netif *netif, struct pbuf *p)`：链路层发包函数，作为 `netif->linkoutput`
- `err_t myif_output(struct netif *netif, struct pbuf *p, ip_addr_t *ipaddr)`：网络层发包函数，作为 `netif->output`。该函数被 `ip_output` 调用，函数内最终会使用 `myif_link_output` 进行发包。若支持 ARP，则该函数可以直接设为 `etharp_output`。
- `myif_input()`：收包函数。当网卡收到包的时候，通过这个函数调用 `netif->input`，将包送入协议栈。对于以太网网卡，`netif->input` 将被设为 `ethernet_input`，故调用 `netif->input` 时需要传递含有数据链路层头部信息的以太网帧。
- `u32_t sys_now(void)`：获取当前时钟，用于实现定时器。
- `main()`：裸机运行的主函数，依次初始化协议栈（`lwip_init()`），添加网卡（`netif_add()`，添加时会初始化），然后进入循环，不断检查收包（收到则调用 `myif_input()`），检查定时器（`sys_check_timeouts()`）

##### netif 初始化时需要设置的字段

- `state`：可选，自定义数据，可用来指向驱动中对网卡的包装结构的地址
- `hwaddr_len`：链路层地址长度
- `hwaddr[]`：链路层地址
- `mtu`：MTU
- `name[2]`：网卡名，如 `en`
- `num`：可选数字，网卡名相同时通过该数字区分
- `output`：设为 `etharp_output`
- `link_output`：设为 `myif_link_output`
- `input`：设为 `ethernet_input`
- `flags`：网卡 flag

#### 实现

##### 引入 lwip 模块

在 `crate` 中创建模块 `lwip_rust`，目录组织如下：

- `custom/`：移植需要的文件
- `depend/`：以 git submodule 的形式导入的 lwip 库
- `src/`：包装为 rust 模块
- `build.rs`：编译和生成接口脚本，参考 <https://github.com/eycorsican/leaf/blob/b0779107921683204a65bb1d41edc07a52688613/leaf/build.rs>
- `wrapper.h`：所有需要生成接口的头文件

##### 编写移植所需头文件

###### `lwipopts.h`

参考：

- <https://github.com/eycorsican/lwip-leaf/blob/12db774b78541b16d448ac58354d326536b79003/custom/lwipopts.h>
- <https://lwip.fandom.com/wiki/Porting_For_Bare_Metal>

主要内容：

- `NO_SYS`
- 各项功能是否开启
- 各项参数（内存、TCP 等）
- 调试开关
- 数据统计开关

###### `arch/cc.h`

- 若 libc 缺少一些头文件，则在这里关闭对应宏
- 调试信息输出函数的相关定义：

  ```c
  #define lwip_NO_INTTYPES_H 1
  #define U8_F               "hhu"
  #define S8_F               "hhd"
  #define X8_F               "hhx"
  #define U16_F              "hu"
  #define S16_F              "hd"
  #define X16_F              "hx"
  #define U32_F              "u"
  #define S32_F              "d"
  #define X32_F              "x"
  #define SZT_F              "zu"
  
  extern int lwip_print(const char *fmt, ...);
  extern void lwip_abort();
  
  #define LWIP_PLATFORM_DIAG(x) \
      do {                      \
          lwip_print x;         \
      } while (0)
  
  #define LWIP_PLATFORM_ASSERT(x)                                                       \
      do {                                                                              \
          lwip_print("Assert \"%s\" failed at line %d in %s\n", x, __LINE__, __FILE__); \
          lwip_abort();                                                                 \
      } while (0)
  ```

  然后在 rust 中 `printf-compat` crate 补充函数的实现：

  ```rust
  #[no_mangle]
  unsafe extern "C" fn lwip_print(str: *const c_uchar, mut args: ...) -> c_int {
      use printf_compat::{format, output};
      let mut s = String::new();
      let bytes_written = format(
          str as *const cty::c_char,
          args.as_va_list(),
          output::fmt_write(&mut s),
      );
      let now = current_time();
      let cpu_id = this_cpu_id();
      ax_print!(
          "[{:>3}.{:06} {}] {}",
          now.as_secs(),
          now.subsec_micros(),
          cpu_id,
          s
      );
      bytes_written
  }
  
  #[no_mangle]
  extern "C" fn lwip_abort() {
      panic!("lwip_abort");
  }
  ```

###### `arch/sys_arch.h`

- 对于 NO_SYS 模式，做一些定义

- 一些编译器库中没有的函数也顺便在这个头文件中定义

```c
#ifndef __ARCH_SYS_ARCH_H__
#define __ARCH_SYS_ARCH_H__

#define SYS_MBOX_NULL NULL
#define SYS_SEM_NULL  NULL

#define isspace(a) ((a == ' ' || (unsigned)a - '\t' < 5))
#define isdigit(a) (((unsigned)(a) - '0') < 10)

int strcmp(const char *l, const char *r);

#endif /* __ARCH_SYS_ARCH_H__ */
```

##### 与驱动对接

实现在 `modules/axnet/src/lwip_impl/driver.rs`

主要学习 `smoltcp_impl` 对 `NetDevices` 的使用。

先实现两个包装：

```rust
struct DeviceWrapper {
    inner: RefCell<AxNetDevice>,
    rx_buf_queue: VecDeque<NetBufferBox<'static>>,
}

struct InterfaceWrapper {
    name: &'static str,
    dev: Arc<Mutex<DeviceWrapper>>,
    netif: Mutex<NetifWrapper>,
}
```

`DeviceWrapper` 复用 `smoltcp_impl` 的实现。

对于收包，对 `InterfaceWrapper` 实现 `poll` 函数，处理收包，并传递给 `netif->input`。

对于发包，实现发包函数 `ethif_output`，将 `pbuf` 复制为 `tx_buf` 然后发包。

对于 `NO_SYS` 模式，提供一个不断调用的循环主函数，用于处理轮询和定时器事件：

``` rust
pub fn lwip_loop_once() {
    let guard = LWIP_MUTEX.lock();
    unsafe {
        ETH0.poll();
        sys_check_timeouts();
    }
    drop(guard);
}
```

### TCP 支持

#### 分析

需要实现的接口：

- `IpAddr`
  - `pub fn from_str(s: &str) -> Result<IpAddress>`

- `Ipv4Addr`
- `SocketAddr`
- `TcpSocket`
  - `pub fn new() -> Self`
  - `pub fn local_addr(&self) -> AxResult<SocketAddr>`
  - `pub fn peer_addr(&self) -> AxResult<SocketAddr>`
  - `pub fn connect(&mut self, _addr: SocketAddr) -> AxResult`
  - `pub fn bind(&mut self, _addr: SocketAddr) -> AxResult`
  - `pub fn listen(&mut self) -> AxResult`
  - `pub fn accept(&mut self) -> AxResult<TcpSocket>`
  - `pub fn shutdown(&self) -> AxResult`
  - `pub fn recv(&self, _buf: &mut [u8]) -> AxResult<usize>`
  - `pub fn send(&self, _buf: &[u8]) -> AxResult<usize>`
  - `fn drop(&mut self) {}`
- `pub(crate) fn init(_net_devs: NetDevices)`

#### 实现

##### 学习 lwip 的 tcp 操作

主要参考：<https://www.nongnu.org/lwip/2_1_x/group__tcp__raw.html>

##### 定义 `TcpSocket` 的结构

```rust
struct TcpPcbPointer(*mut tcp_pcb);
unsafe impl Send for TcpPcbPointer {}
struct PbuffPointer(*mut pbuf);
unsafe impl Send for PbuffPointer {}

struct TcpSocketInner {
    remote_closed: bool,
    connect_result: i8,
    recv_queue: Mutex<VecDeque<PbuffPointer>>,
    accept_queue: Mutex<VecDeque<TcpSocket>>,
}

pub struct TcpSocket {
    pcb: TcpPcbPointer,
    inner: Pin<Box<TcpSocketInner>>,
}
```

lwip 以回调函数的形式执行各种操作。

为了可以在回调函数中获取到对应的 `TcpSocket` 里的内容，在初始化时将自定义的传给回调函数的参数设为 `inner` 的地址。

`TcpSocket` 本身地址可能发生变化，故使用 `Pin<Box<T>>` 创建 `inner`，将地址固定住。

##### 实现 `TcpSocket`

以 `connect` 为例：

- 在做任何 lwip 协议栈操作之前，获取 `LWIP_MUTEX` 协议栈锁
- 使用 `inner` 中存储的状态判断操作是否完成，若无法完成则 `lwip_loop_once` 并 `yield_now`，直到协议栈调用回调函数修改 `inner`

```rust
extern "C" fn connect_callback(arg: *mut c_void, _tpcb: *mut tcp_pcb, err: err_t) -> err_t {
    debug!("[TcpSocket] connect_callback: {:#?}", err);
    let socket_inner = unsafe { &mut *(arg as *mut TcpSocketInner) };
    socket_inner.connect_result = err;
    err
}

pub fn connect(&mut self, addr: SocketAddr) -> AxResult {
    debug!("[TcpSocket] connect to {:#?}", addr);
    let ip_addr: ip_addr_t = addr.addr.into();
    self.inner.connect_result = 1;

    // lock lwip
    let guard = LWIP_MUTEX.lock();
    unsafe {
        debug!("[TcpSocket] set recv_callback");
        tcp_recv(self.pcb.0, Some(recv_callback));

        debug!("[TcpSocket] tcp_connect");
        #[allow(non_upper_case_globals)]
        match tcp_connect(self.pcb.0, &ip_addr, addr.port, Some(connect_callback)) as i32 {
            err_enum_t_ERR_OK => {}
            err_enum_t_ERR_VAL => {
                return ax_err!(InvalidInput, "LWIP [tcp_connect] Invalid input.");
            }
            _ => {
                return ax_err!(Unsupported, "LWIP [tcp_connect] Failed.");
            }
        };
    }
    drop(guard);

    // wait for connect
    debug!("[TcpSocket] wait for connect");
    lwip_loop_once();
    #[allow(clippy::while_immutable_condition)]
    while self.inner.connect_result == 1 {
        yield_now();
        lwip_loop_once();
    }
    debug!("[TcpSocket] connect result: {}", self.inner.connect_result);

    if self.inner.connect_result == 0 {
        Ok(())
    } else {
        ax_err!(Unsupported, "LWIP [connect_result] Unsupported")
    }
}
```

其余操作也类似，如果需要提供阻塞的接口，则根据 `inner` 中的结果 / 队列进行阻塞，直到回调函数将 `inner` 中的对应部分进行修改。

### UDP / DNS 支持

#### UDP

学习 lwip 的 UDP raw api：

- <https://lwip.fandom.com/wiki/Raw/UDP>
- <https://www.nongnu.org/lwip/2_1_x/group__udp__raw.html>

使用 lwip 中的：

- `udp_new`
- `udp_bind`
- `udp_sendto`
- `udp_recv`
- `udp_remove`

完成 `UdpSocket`：

- [x] `UdpSocket`
  - [x] `pub fn new() -> Self`
  - [x] `pub fn local_addr(&self) -> AxResult<SocketAddr>`
  - [x] `pub fn bind(&mut self, addr: SocketAddr) -> AxResult`
  - [x] `pub fn sendto(&self, buf: &[u8], addr: SocketAddr) -> AxResult<usize>`
  - [x] `pub fn recvfrom(&self, buf: &mut [u8]) -> AxResult<(usize, SocketAddr)>`
  - [x] `pub fn shutdown(&mut self) -> AxResult`
  - [x] `fn drop(&mut self)`

适配方式与 TcpSocket 类似。

#### DNS

学习 lwip 的 DNS raw api：

- <https://lwip.fandom.com/wiki/DNS>
- <https://www.nongnu.org/lwip/2_1_x/group__dns.html>

使用 lwip 中的：

- `dns_setserver`：设置 DNS 服务器，暂时硬编码为 8.8.8.8
- `err_t dns_gethostbyname(const char *hostname, ip_addr_t *addr, dns_found_callback found, void *callback_arg)`：非阻塞查询 DNS，结果如下
  - `ERR_OK`：命中缓存，查询结果存放在 `addr` 中
  - `ERR_INPROGRESS`：进行查询，查询完成后调用回调函数 `found`
  - `ERR_VAL`：未设定 DNS 服务器地址或其他错误
  - `ERR_ARG`：参数错误或其他错误

完成阻塞查询函数 `pub fn resolve_socket_addr(name: &str) -> AxResult<Vec<IpAddr>>`

适配方式与 TcpSocket 类似。

### IPv6 支持

补全一 IPv6 所需的部分：

``` diff
+   netif_create_ip6_linklocal_address(&mut ETH0.netif.lock().0, 1);
    netif_set_link_up(&mut ETH0.netif.lock().0);
    netif_set_up(&mut ETH0.netif.lock().0);
    netif_set_default(&mut ETH0.netif.lock().0);
```

``` diff
    (*netif).output = Some(etharp_output);
+   (*netif).output_ip6 = Some(ethip6_output);
    (*netif).linkoutput = Some(ethif_output);
```

创建 `Ipv6Addr` 结构体，以及相关的类型转换和输出。

在 init 时输出地址，如：

``` plain
created net interface "eth0":
  ether:    52-54-00-12-34-56
  ip:       10.0.2.15/24
  gateway:  10.0.2.2
  ip6:      fe80::5054:ff:fe12:3456
```

### 构建与测试

这部分十分耗时，迭代多个版本，此处仅呈现最终结果。

#### lwip_rust

引入 lwip 库后，需要进行编译，并生成供 rust 调用的接口。

##### 编译

使用 crate `cc`，在 `build.rs` 里将 lwip 编译为静态库 `liblwip.a`

```rust
let mut base_config = cc::Build::new();
```

头文件目录分别为 lwip 的 include 目录、用于移植的头文件目录：

```rust
base_config
    .include("depend/lwip/src/include")
    .include("custom");
```

然后将所有需要的源文件导入：

```rust
base_config.file("depend/lwip/src/core/xxx.c");
```

最后定义编译参数并编译：

```rust
base_config
    .warnings(true)
    .flag("-static")
    .flag("-no-pie")
    .flag("-fno-builtin")
    .flag("-ffreestanding")
    .compile("liblwip.a");
```

编译时还需要指定使用的 `TARGET_CC` 和 `TARGET_CFLAGS`，用于选择编译器，以及是否使用浮点数等。此处在 Makefile 中指定。

##### 生成 Rust 接口

使用 crate `bindgen` 生成接口，`wrapper.h` 中包含了所有需要生成接口的头文件：

```rust
let bindings = bindgen::Builder::default()
    .use_core()
    .header("wrapper.h")
    .clang_arg("-I./depend/lwip/src/include")
    .clang_arg("-I./custom")
    .clang_arg("-Wno-everything")
    .layout_tests(false)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");
```

最后输出 `bindings.rs` 到 `src/` 中供 `lib.rs` 使用：

```rust
let out_path = PathBuf::from("src");
bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
```

#### 构建

- make 时 export 出对应的 `TARGET_CC` 和 `TARGET_CFLAGS`
- 添加 `NET_DEV` 用于指定网络后端，默认为 `user`，可以改为 `tap`
- `NET_DUMP` 指定是否需要抓包
- 在 `APP_FEATURES` 中添加 `libax/use-lwip` 指定开启 lwip，否则使用 smoltcp

#### CI

添加递归签出，获取 lwip submodule：

``` yaml
- uses: actions/checkout@v3
    with:
    submodules: recursive
```

在必要时添加 musl 工具链的下载：

- 尽量仅在编译时下载工具链
- 当 `CARGO_CFG_TARGET_OS` 不为 `none` 时（生成 doc 时）不编译
- 当 `CLIPPY_ARGS` 存在时（clippy 时）不编译
- 对于 unit_test，添加参数 `--exclude lwip_rust` 避免编译

具体实现：

``` rust
let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
let clippy_args = std::env::var("CLIPPY_ARGS");

// Not build with clippy or doc
if target_os == "none" && clippy_args.is_err() {
    compile_lwip();
}
generate_lwip_bindings();
```

在缺少 musl 工具链时进行 bindgen：

- CI 中添加 `sudo apt update && sudo apt install -y gcc-multilib`

#### 测试

对于所有网络相关测试，添加一份 `APP_FEATURES` 增加 `libax/use-lwip` 的测试。

### 各种问题与调试

一部分较为值得记录的问题调试。

#### 链接脚本问题

链接脚本的 `.bss` 段内缺少 `*(COMMON)` 段，`.rodata` 段内缺少 `*(.sdata2 .sdata2.*)` 段，故如下修改：

``` diff
.rodata : ALIGN(4K) {
    srodata = .;
    *(.rodata .rodata.*)
    *(.srodata .srodata.*)
+   *(.sdata2 .sdata2.*)
    . = ALIGN(4K);
    erodata = .;
}
```

``` diff
.bss : ALIGN(4K) {
    boot_stack = .;
    *(.bss.stack)
    . = ALIGN(4K);
    boot_stack_top = .;

    sbss = .;
    *(.bss .bss.*)
    *(.sbss .sbss.*)
+   *(COMMON)
    . = ALIGN(4K);
    ebss = .;
}
```

调试历程：

- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week8.md#%E9%93%BE%E6%8E%A5%E8%84%9A%E6%9C%AC%E9%97%AE%E9%A2%98>
- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week9.md#%E9%93%BE%E6%8E%A5%E8%84%9A%E6%9C%AC%E9%97%AE%E9%A2%98>
- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week12.md#%E9%93%BE%E6%8E%A5%E8%84%9A%E6%9C%AC%E9%97%AE%E9%A2%98>

#### cargo 构建问题

`dependencies` 和 `build-dependencies` 的 `features` 混合造成混乱，通过在 `[workspace]` 中指定 `resolver = "2"` 解决。

调试历程：

- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week8.md#cargo-%E6%9E%84%E5%BB%BA%E9%97%AE%E9%A2%98>

#### apache benchmark 测试卡住问题

未完全解决，使用 TAP 作为网络后端可一定程度上缓解。

调试历程：

- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week10.md#debug>
- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week12.md#ab-%E6%B5%8B%E8%AF%95%E5%8D%A1%E4%BD%8F%E9%97%AE%E9%A2%98>

#### `int8_t` 应为 `signed char` 而不是 `char`

标准中并未确定 char 是否有符号，而是将其取决于实现。故 `int8_t` 应明确指定 `signed char` 实现。

调试历程：

- <https://github.com/Centaurus99/arceos-lwip/blob/main/reports/week11.md#%E8%B8%A9%E5%9D%91>



## 性能优化

### 内存拷贝优化

#### 分析

分析 lwip 数据处理流程：

``` plain
+----------+    Rx (Copy)   +-----------+    Rx (Copy)   +-----------+
|          +--------------->|           +--------------->|           |
|  Driver  |                | Net Stack |                |    App    |
|          |<---------------+           |<---------------+           |
+----------+    Tx (Copy)   +-----------+    Tx (Copy)   +-----------+
```

由于 socket 接口不变，故 App 和 Net Stack 间的拷贝无法消除。

对于发包时向 Driver 的拷贝，由于 Driver 提供的发包接口中，传递的内存段前部要求预留 `VirtIOHeader` 的空间，而 lwip 提供的与驱动的接口无法在前部预留空间故想要消除这部分的拷贝，则需要对 Driver 的接口进行修改。

对于收包时向 lwip 的拷贝，可以通过使用 `PBUF_REF` 类型的 `pbuf_custom`，自定义析构函数，从而 lwip 可以直接使用 Driver 提供的存储包数据的内存块进行处理，无需拷贝。

综上，可以优化为如下情况（与 smoltcp 的拷贝情况相同）：

``` plain
+----------+ Rx (Zero-Copy) +-----------+    Rx (Copy)   +-----------+
|          +--------------->|           +--------------->|           |
|  Driver  |                | Net Stack |                |    App    |
|          |<---------------+           |<---------------+           |
+----------+    Tx (Copy)   +-----------+    Tx (Copy)   +-----------+
```

#### 数据

在 `memcpy` 函数内统计 lwip 和 smoltcp 的拷贝次数和长度。

对 `app/c/httpclient` 的 10000 个请求：

|             | memcpy 次数 | memcpy 总长度 |
| :---------: | :---------: | :-----------: |
|    lwip     |  约 190000  |   11485520    |
|   smoltcp   |  约 140000  |   21812664    |
| lwip 优化后 |  约 140000  |    7764834    |

lwip 在优化后达到了与 smoltcp 相似的拷贝次数，拷贝总长度约为 smoltcp 的三分之一。

#### 实现

在收包时创建对应的 `pbuf_custom`，指向 Driver 分配的内存段，设置自定义的析构函数 `extern "C" fn pbuf_free_custom(p: *mut pbuf)`，这样 lwip 在处理完成之后调用析构函数，回到 rust 中回收 Driver 分配的资源。

具体见 commit [c212ccb](https://github.com/Centaurus99/arceos-lwip/commit/c212ccbcfbc2d2a104bda980d34bb37c2e9f08e7)

### 内存分配优化

#### 分析

lwip 的适配过程中，有些地方使用了堆分配内存，这会带来一定的性能开销。在条件允许的情况下，换用内存池分配内存可以提高内存分配性能。

在优化之前，lwip 的适配中进行堆分配的地方有如下部分：

- 收包时创建的 `pbuf_custom`
- 对于每个 TCP 连接创建的 TCP socket
- 收包队列 `recv_queue: VecDeque`
- TCP 的 accept 队列 `accept_queue: VecDeque`

对于在堆中创建 struct 的需求，可以通过在内存池中分配来提高性能。

对于两种队列，可以通过一开始预分配一定空间开减少堆分配次数。

#### 数据统计

在 `alloc` 函数内统计 lwip 和 smoltcp 的堆分配次数和大小。

对 `app/net/httpclient` 的 1000 个请求：

|                              | alloc 次数 | alloc 总大小 |
| :--------------------------: | :--------: | :----------: |
|         lwip 无优化          |   13,015   | 263,729,578  |
|       lwip 内存池分配        |   8,009    | 263,489,290  |
| lwip 内存池分配 + 队列预分配 |   8,009    | 263,473,242  |
|           smoltcp            |   8000+    | 271,000,000+ |

由于 `app/net/httpclient` 对于每个请求会创建新线程，大小 0x4000 的线程栈会在堆中分配，故统计的总大小均很大，主要关注 alloc 次数即可。

实测中由于队列最大长度也很小，故队列预分配效果有限。

lwip 在优化后达到了与 smoltcp 相似的堆分配次数，且总大小显著更少。

#### 实现

使用 lwip 提供的内存池进行分配。

在 `crates/lwip_rust/custom/custom_pool.h` 和 `crates/lwip_rust/custom/custom_pool.c` 中声明内存池，并提供 初始化 / 分配 / 回收 的函数，可以在 rust 中调用。回调函数也做相应的更改。

具体见 commit [a00e67c](https://github.com/Centaurus99/arceos-lwip/commit/a00e67c33b5f465458e1ea75dc73e4ae3da667f8)

### lwip 参数优化

启用 lwip 内置的数据统计开关，统计内存分配信息：

``` c
#define LWIP_STATS         1
#define LWIP_STATS_DISPLAY 1
```

根据测试结果将 内存堆 / 内存池 开到够用大小，主要调整参数如下：

``` c
// Important performance options
// Smaller values increase performance
// Larger values increase simultaneously active TCP connections limit
#define MEMP_NUM_TCP_PCB 5

// Memory options
#define MEM_SIZE         (32 * 1024)
#define MEMP_NUM_TCP_SEG 16
#define MEMP_NUM_PBUF    32
#define PBUF_POOL_SIZE   32

// Tcp options
#define TCP_MSS     1460
#define TCP_WND     (32 * TCP_MSS)
#define TCP_SND_BUF (16 * TCP_MSS)
```

其中最影响性能的参数为 `MEMP_NUM_TCP_PCB`：

- 该参数限制了协议栈同时处理的 TCP 连接数
- 太小则会导致协议栈无法同时建立足够多的 TCP 连接，新的连接需要等待旧连接关闭后才能建立
- 太大则会在压力测试时导致协议栈同时维护过多处于 `TIME_WAIT` 状态的待关闭连接（这种状态的连接只会在 有新连接要建立 或者 等待时间达到两倍报文段寿命 时才会被最终关闭），影响性能

### 性能热点分析

对于一般的程序，通常使用 perf 工具，画出火焰图进行性能热点分析。但对于 qemu 中运行的 OS，无法使用这样的方式。

另一种方式是统计各函数的运行时间，但稍微麻烦一些。

我使用了一种基于 gdb 不断打 backtrace 的统计调用的方式，来自 <https://poormansprofiler.org/>：

``` shell
#!/bin/bash
nsamples=500
sleeptime=0

for x in $(seq 1 $nsamples)
  do
    gdb-multiarch apps/net/httpserver//httpserver_pc-x86.elf -ex 'target remote localhost:1234' -ex "thread apply all bt" -batch
  done | \
awk '
  BEGIN { s = ""; } 
  /^Thread/ { print s; s = ""; } 
  /^\#/ { if (s != "" ) { s = s "," $4} else { s = $4 } } 
  END { print s }' | \
sort | uniq -c | sort -r -n -k 1,1

```

结果如下：

``` bash
thx@ubuntu181:~/Git/arceos-lwip$ sudo ./perf.sh 
awk: cmd. line:4: warning: regexp escape sequence `\#' is not a known regexp operator
    235 axnet::lwip_impl::driver::ethif_output,ip4_output_if_src,memp_memory_TCP_PCB_base,ram_heap,axhal::platform::pc_x86::dtables::__PERCPU_TSS,inet_cksum_pseudo_partial_base,axhal::platform::pc_x86::dtables::__PERCPU_TSS,??
    132 axnet::lwip_impl::driver::ethif_output,ip4_output_if_src,memp_memory_TCP_PCB_base,ram_heap
    114 axnet::lwip_impl::driver::ethif_output,ip4_output_if_src,memp_memory_TCP_PCB_base,ram_heap,memp_memory_TCP_PCB_base,??,??,axnet::lwip_impl::driver::ETH0,ram_heap,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,ram_heap,tcp_output_control_segment,axnet::lwip_impl::driver::ETH0,tcp_output_control_segment,??,memp_memory_TCP_PCB_base,??,??
     92 axtask::api::yield_now,main
     66 tcp_receive,??,??,dns_pcbs,??,??,??
     55 axnet::lwip_impl::driver::lwip_loop_once,main
     42 main
     39 tcp_alloc,memp_memory_TCP_PCB_LISTEN_base,??,??
     37 axtask::run_queue::AxRunQueue::resched_inner,axtask::api::yield_now,main
     35 tcp_output,memp_memory_TCP_PCB_base,tcp_enqueue_flags
     29 sys_check_timeouts,axhal::platform::pc_x86::boot::BOOT_STACK,axhal::platform::pc_x86::boot::BOOT_STACK,axhal::platform::pc_x86::boot::BOOT_STACK,??,pbuf_alloc_reference,axnet::lwip_impl::driver::lwip_loop_once,main
     26 axtask::wait_queue::WaitQueue::notify_one,axnet::lwip_impl::driver::lwip_loop_once,main
     11 sys_now,sys_check_timeouts,axhal::platform::pc_x86::boot::BOOT_STACK,axhal::platform::pc_x86::boot::BOOT_STACK,axhal::platform::pc_x86::boot::BOOT_STACK,??,pbuf_alloc_reference,axnet::lwip_impl::driver::lwip_loop_once,main
     11 axsync::mutex::Mutex<T>::lock,axnet::lwip_impl::driver::lwip_loop_once,main
     10 tcp_output,??
     10 axtask::wait_queue::WaitQueue::notify_one,main
      9 <axsync::mutex::MutexGuard<T>,axnet::lwip_impl::driver::lwip_loop_once,main
      8 tcp_output,sdata,axhal::platform::pc_x86::dtables::__PERCPU_TSS,??,??,axhal::platform::pc_x86::dtables::__PERCPU_TSS
      7 tcp_pcb_remove,axhal::platform::pc_x86::dtables::__PERCPU_TSS,tcp_tw_pcbs,tcp_abandon,??
      5 axsync::mutex::Mutex<T>::lock,main
      5 <axsync::mutex::MutexGuard<T>,main
      5 axnet::lwip_impl::driver::ethif_output,ip4_output_if_src,memp_memory_TCP_PCB_base,ram_heap,memp_memory_TCP_PCB_base,??,axhal::platform::pc_x86::dtables::__PERCPU_TSS,axnet::lwip_impl::driver::ETH0,ram_heap,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,ram_heap,tcp_output_control_segment,axnet::lwip_impl::driver::ETH0,tcp_output_control_segment,??,memp_memory_TCP_PCB_base,??,??
      4 tcp_pcbs_sane,memp_memory_TCP_PCB_base,??,??
      3 sys_check_timeouts,axnet::lwip_impl::driver::lwip_loop_once,main
      3 axnet::lwip_impl::driver::ethif_output,ip4_output_if_src,memp_memory_TCP_PCB_base,ram_heap,ram_heap,??,SELF_PTR,axnet::lwip_impl::driver::ETH0,ram_heap,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,ram_heap,tcp_output_control_segment,axnet::lwip_impl::driver::ETH0,tcp_output_control_segment,SELF_PTR,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,??
      2 axnet::lwip_impl::driver::ethif_output,ip4_output_if_src,memp_memory_TCP_PCB_base,ram_heap,ram_heap,??,SELF_PTR,axnet::lwip_impl::driver::ETH0,ram_heap,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,memp_memory_TCP_PCB_base,ram_heap,tcp_output_control_segment,axnet::lwip_impl::driver::ETH0,tcp_output_control_segment,??,memp_memory_TCP_PCB_base,??,SELF_PTR,dns_pcbs,memp_memory_TCP_PCB_base,SELF_PTR,tcp_send_empty_ack,SELF_PTR,tcp_input,??
      1 tcp_pcbs_sane,??,??,??
      1 tcp_input,??,??
      1 tcp_abort,tcp_alloc,memp_memory_TCP_PCB_LISTEN_base,??,??
      1 memp_malloc,tcp_alloc,memp_memory_TCP_PCB_LISTEN_base,??,??
      1 <allocator::slab::SlabByteAllocator,alloc::sync::Arc<T>::drop_slow,axtask::run_queue::gc_entry,core::ops::function::FnOnce::call_once{{vtable-shim}},axtask::task::task_entry,??
      1 
```

可粗略发现处于 `ethif_output` 的次数非常多，提示这儿可能是性能热点。

思考后发现，这也十分合理：当前发包时协议栈需要阻塞等待驱动完成发包，在驱动完成发包前，协议栈啥也干不了。

所以预计改为非阻塞可较大程度提升性能。

### lwip 其他优化分析

- 官方优化指南：<https://lwip.fandom.com/wiki/Maximizing_throughput>
  - 大端系统优于小端系统
  - 驱动程序可能是瓶颈
  - TCP / UDP 校验和计算很可能是瓶颈，建议硬件计算
  - memcpy 可以优化（尽可能增加复制时的字长）
  - 提高内存大小，尽可能使用内存池
  - 若无法硬件计算校验和，则启用 `LWIP_CHECKSUM_ON_COPY` 在 memcpy 时计算校验和
  - ……
- 官方 TCP 调参指南：<https://lwip.fandom.com/wiki/Tuning_TCP>
  - TCP_MSS 尽可能高（目前已设为 1460）
  - TCP_WND 尽可能高，受内存限制
  - 启用 `TCP_QUEUE_OOSEQ` 乱序收包
  - TCP_SND_BUF 应设为 TCP_WND 相同值
  - TCP_OVERSIZE 设为 TCP_MSS，发送时只申请一个 pbuf，效率更高
  - ……
- 其他优化：
  - 内存分配优化：[Improvement and Optimization of LwIP](https://ieeexplore.ieee.org/document/7867431)
    - 针对嵌入式场景，应用简单，CPU 和内存资源极其有限
    - 使用一个内存池和一个内存堆，统一管理，内存池的分配大小和容量预先根据对应用的测量数据进行调整
    - 参考意义不大
  - 协议栈设计
    - 大量相关文献



## 性能测试

系统版本：`Ubuntu 20.04.3 LTS`

内核版本：`5.4.0-125-generic`

CPU：`Intel(R) Xeon(R) Gold 6230 CPU @ 2.10GHz`

QEMU：`QEMU emulator version 8.0.0`

运行参数：

- `make A=apps/net/httpserver/ ARCH=riscv64 LOG=warn NET=y NETDEV=tap MODE=release run`
- `qemu-system-riscv64 -m 128M -smp 1 -machine virt -bios default -kernel apps/net/httpserver//httpserver_qemu-virt-riscv.bin -device virtio-net-device,netdev=net0 -netdev tap,id=net0,ifname=qemu-tap0,script=no,downscript=no -nographic`

测试参数：`ab -n 100000 -c 100 http://10.0.2.15:5555/`
