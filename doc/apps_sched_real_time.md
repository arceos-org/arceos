# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [sched-realtime](../apps/task/sched-realtime/) | axalloc, axtask | alloc, paging, multitask, sched_rr, sched_fifo, sched_mlfq, sched_sjf, sched_cfs | schedule test short payload & yield|

# RUN
## CFS
First, modify the dependencies in Cargo.toml:
```
...
[dependencies]
libax = { path = "../../../ulib/libax", default-features = false, features = ["alloc", "paging", "multitask", "sched_cfs"] }
```
Then make the code.
```shell
make A=apps/task/sched-realtime LOG=info APP_FEATURES=sched_cfs run
```
## Other choises

First, modify the dependencies in Cargo.toml:
```
...
[dependencies]
libax = { path = "../../../ulib/libax", default-features = false, features = ["alloc", "paging", "multitask"] }
```
Then make the code.
```shell
make A=apps/task/sched-realtime LOG=info APP_FEATURES=sched_rr run
Other choises of APP_FEATURES: sched_fifo, sched_mlfq, sched_sjf
```

## Using multicore
```shell
make A=apps/task/sched-realtime LOG=info APP_FEATURES=... SMP=4 run
```

# RESULT
```
make A=apps/task/sched-realtime LOG=info APP_FEATURES=sched_mlfq SMP=4 run
...
part 4: TaskId(7) [0, 8)
part 3: TaskId(8) [0, 500000)
part 0: TaskId(11) [0, 50000)
part 2: TaskId(9) [0, 200000)
part 1: TaskId(10) [0, 100000)
part 4: TaskId(7) finished
part 0: TaskId(11) finished
part 3: TaskId(8) finished
part 1: TaskId(10) finished
part 2: TaskId(9) finished
main task woken up! timeout=false
maximum leave time = 2260ms
sum = 40000006431895672
leave time = 254ms, 392ms, 531ms, 695ms, 2260ms
Parallel summation tests run OK!
[  5.585371 2:2 axhal::platform::qemu_virt_riscv::misc:2] Shutting down...
```
# PROCESS

### 各算法参数设置

- cfs：默认情况下所有的 nice 值都是 0，此时期望是完全公平调度。
- sjf：移动平均系数默认设为 1/16，初值是 0
- mlfq：队列级数为 8，第 0 级队列分到 1 个时间片，过 100000 个时间片重置队列。同优先级抢占式调度。
- rr：MAX_TIME_SLICE=5

注：如果没有特殊说明，默认在**单核**模式下测试，时间单位为 **ms**。如果 cfs 设置了 nice 值，会额外说明。

### 设计思路：

有 4 个短任务，yield 次数分别是 500000,100000,200000，500000，循环长度分别是 10, 5, 2, 1。有一个长任务，yield 次数 8，循环长度 100000000。

- 注：对于所有调度器来说 yield 都较慢，不要直接用 yield 次数 乘以 循环长度估算时间。实际上长任务的运行时间约为所有短任务之和的两倍。

## COMPARATION

|      | 长任务完成时间 | 短任务1完成时间 | 短任务2完成时间 | 短任务3完成时间 | 短任务4完成时间 |
| ---- | -------------- | --------------- | --------------- | --------------- | --------------- |
| rr   | 2031           | 2143            | 2275            | 2432            | 2619            |
| fifo | 2012           | 1856            | 1924            | 2014            | 2100            |
| cfs  | 2805           | 168             | 311             | 496             | 1038            |
| sjf  | 2742           | 630             | 719             | 588             | 422             |
| mlfq | 2399           | 381             | 440             | 523             | 623             |



## ANALYSIS

- rr 和 fifo 实时性很差，rr 的短任务的结束时间远远晚于长任务结束时间，fifo 的短任务结束时间和长任务结束时间近似。
- cfs 忠实地按照公平分配原则。由于 yield 实际上相比不超过 10 次的循环占主要时间，所以几个短任务的时间大致和 yield 次数成正比（其实预期也不完全是正比，推一下式子本来就是大致正比）。得到了较好的实时性。
- sjf 按照谁快谁先的原则，因为这几个论速度差不多快，所以最早结束的短任务时间略晚于 cfs。不过短任务总的结束时间比 cfs 早，体现了较好的实时性。
- mlfq 短任务会有更高的优先级，抢占式调度，有较好的实时性。

#### 针对 cfs nice 值的设置
- 全部设置为 0：同上
  
|      | 长任务完成时间 | 短任务1完成时间 | 短任务2完成时间 | 短任务3完成时间 | 短任务4完成时间 |
| ---- | -------------- | --------------- | --------------- | --------------- | --------------- |
| cfs0 | 2805           | 168             | 311             | 496             | 1038            |

- 将短任务全部设为 5，长任务设为 -5，如下
  
|      | 长任务完成时间 | 短任务1完成时间 | 短任务2完成时间 | 短任务3完成时间 | 短任务4完成时间 |
| ---- | -------------- | --------------- | --------------- | --------------- | --------------- |
| cfs1  | 2407           | 403             | 1008             | 1887             | 2372            |

- 可以看到，这样的设置可以让 cfs 调度算法更早完成长任务，同时实时性下降
- 将短任务全部设为 5，长任务设为 -5，如下

|      | 长任务完成时间 | 短任务1完成时间 | 短任务2完成时间 | 短任务3完成时间 | 短任务4完成时间 |
| ---- | -------------- | --------------- | --------------- | --------------- | --------------- |
| cfs2  | 2821           | 136             | 248             | 455             | 779            |

- 此时 cfs 具有非常高的实时性。
- 将 2,3,4 号短任务设为 -5, 1 号短任务和长任务都设为 5，如下

|      | 长任务完成时间 | 短任务1完成时间 | 短任务2完成时间 | 短任务3完成时间 | 短任务4完成时间 |
| ---- | -------------- | --------------- | --------------- | --------------- | --------------- |
| cfs3  | 2660           | 575             | 214             | 428             | 738            |

- 此时短任务 1 的实时性下降，其他短任务的实时性上升。