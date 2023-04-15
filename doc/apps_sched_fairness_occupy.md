# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [sched-occupy](../apps/task/sched-occupy/) | axalloc, axtask | alloc, paging, multitask, sched_fifo | short tasks occupy long tasks test|

# RUN
```shell
make A=apps/task/sched-occupy LOG=info APP_FEATURES=sched_cfs run

Other choises of APP_FEATURES: sched_rr, sched_fifo, sched_mlfq, sched_sjf
```

## Using multicore
```shell
make A=apps/task/sched-occupy LOG=info APP_FEATURES=... SMP=4 run
```

# RESULT
```
make A=apps/task/sched-occupy LOG=info APP_FEATURES=sched_mlfq SMP=4 run
...
part 3: TaskId(8) [0, 2000000)
part 4: TaskId(7) [0, 80)
part 2: TaskId(9) [0, 2000000)
part 1: TaskId(10) [0, 0)
part 1: TaskId(10) finished
part 0: TaskId(11) [0, 0)
part 0: TaskId(11) finished
part 4: TaskId(7) finished
main task woken up! timeout=false
long task leave time = 2444ms
Parallel summation tests run OK!
[  3.275980 0:2 axhal::platform::qemu_virt_riscv::misc:2] Shutting down...
```
# PROCESS

### 各算法参数设置

- cfs：默认情况下所有的 nice 值都是 0，此时期望是完全公平调度。
- sjf：移动平均系数默认设为 1/16，初值是 0
- mlfq：队列级数为 8，第 0 级队列分到 1 个时间片，过 100000 个时间片重置队列。同优先级抢占式调度。
- rr：MAX_TIME_SLICE=5

注：如果没有特殊说明，默认在**单核**模式下测试，时间单位为 **ms**。如果 cfs 设置了 nice 值，会额外说明。

### 设计思路

设计思路：有 2 个短进程，运行很多次；有一个长进程，运行 80 次，每次 10000000 长度的循环。测试长进程什么时候结束，反映在有实时任务的情况下的**响应时间**。

## COMPARASION
|      | 长任务完成时间 |
| ---- | -------------- |
| rr   | 2023           |
| fifo | 2019           |
| cfs  | 4828           |
| sjf  | 6740           |
| mlfq | 5416           |

可以看出：rr 和 fifo 响应时间短，cfs 响应时间一般，sjf 响应时间长。

- 这里 mlfq 的响应时间也很长，这和时间片大小和重置时间大小有关系。