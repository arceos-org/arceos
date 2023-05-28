# 第十五周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

### 修 bug

#### TCP Recv 问题

使用 `make A=apps/c/httpserver/ LOG=debug NET=y ARCH=aarch64 MODE=debug APP_FEATURES=libax/use-lwip NETDEV=tap run`。

跑一次 `ab -n 1000 -c 100 http://10.0.2.15:5555/` 之后无法再连接。

打了一晚上的 log，发现问题是 tcp recv 没有正确 close，有可能卡住。

``` diff
- if self.inner.remote_closed {
-     return Ok(0);
- }
loop {
+   if self.inner.remote_closed {
+       return Ok(0);
+   }
    ...
}
```

### 调参

启用 `LWIP_STATS` 和 `LWIP_STATS_DISPLAY`，根据统计数据调整 lwip 各项参数，主要如下：

``` c
// Memory options
#define MEM_SIZE         (1 * 1024 * 1024)
#define MEMP_NUM_TCP_PCB 1024
#define MEMP_NUM_TCP_SEG 1024
#define MEMP_NUM_PBUF    512
#define PBUF_POOL_SIZE   32

// Tcp options
#define TCP_MSS     1460
#define TCP_WND     (32 * TCP_MSS)
#define TCP_SND_BUF (16 * TCP_MSS)
```

## 下周计划
