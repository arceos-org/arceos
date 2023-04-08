# 第七周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

### 分析网络驱动的运行方式

- `axruntime` 在初始化时调用 `axdriver::init_drivers` 来初始化各个设备和驱动（每种类型的设备目前似乎只能有一个？）
- 具体的驱动实现在外部模块 [virtio-drivers](https://github.com/rcore-os/virtio-drivers) 中。对于 net，初始化时会创建 `send_queue` 和 `recv_queue` 这两个 DMA Descriptor Ring，默认长度 64，同时为 `recv_queue` 中的每个 descriptor 创建 buffer（`rx_buffers`），buffer 默认长度 2048。然后包装为 `VirtIoNetDev` 并实现了 trait `NetDriverOps` 供上层使用。
- 接着通过 `axnet::init_network` 对网卡做一些初始化。`axnet` 将 `VirtIoNetDev` 包装为 `DeviceWrapper`，并对其实现了 smoltcp 的 `Device` trait，供 smoltcp 控制设备收发以太网帧。在 receive 时返回 `AxNetRxToken` 和 `AxNetTxToken`，transmit 时返回 `AxNetTxToken`。在 `RxToken` 被 consume 时，将 `RxToken` 里对应 buffer 的包做相应处理，然后回收进网卡驱动中的收包队列；在 `TxToken` 被 consume 时，让网卡驱动创建发包的 buffer，然后向其中填充数据并发送。

### 初步性能测试

系统版本：`Ubuntu 20.04.5 LTS`

内核版本：`Linux 5.15.90.1-microsoft-standard-WSL2`

CPU：`13th Gen Intel(R) Core(TM) i5-13600K`

QEMU：`QEMU emulator version 7.0.0`

**注意：**大小核 CPU，WSL 环境，QEMU 模拟，未绑核，睿频开启，测试环境可控性较差，测试结果仅供初步参考

启动 `httpserver`： `make A=apps/net/httpserver/ ARCH=riscv64 LOG=warn NET=y SMP=$SMP run`

Benchmark：`ab -n 10000 -c $Concurrency http://127.0.0.1:5555/`

Requests per second 取三次测试平均值

| SMP ↓  \|  Concurrency →  \|  Requests per second ↘ |    1     |    2     |    4     |    8     |    16    |
| :-------------------------------------------------: | :------: | :------: | :------: | :------: | :------: |
|                          1                          | 7753.343 | 8002.776 | 7943.860 | 7774.110 | 7199.413 |
|                          2                          | 7636.490 | 7863.210 | 7951.306 | 7865.323 | 7383.263 |
|                          4                          | 7511.503 | 7670.896 | 7905.193 | 7470.166 | 6477.220 |
|                          8                          | 7420.986 | 7600.253 | 7616.513 | 7544.476 | 6902.583 |

观察：

- 高并发下 longest request time 变长，且可能有连接会卡住。
- 增加 SMP 不会带来性能提升，反而有性能降低的可能。

## 下周计划

- 学习 `lwip` 对底层驱动的使用方式
- 尝试将 `lwip` 以 C App 的形式进行接入
- ……

