# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [sched_fairness_unequal](../apps/task/sched_fairness_unequal/) | axalloc, axtask | alloc, paging, multitask, sched_rr, sched_fifo, sched_mlfq, sched_sjf, sched_cfs | schedule test unequal payload|

# RUN
```shell
make A=apps/task/sched-fairness-unequal LOG=info APP_FEATURES=sched_cfs run

Other choises of APP_FEATURES: sched_rr, sched_fifo, sched_mlfq, sched_sjf
```

## Using multicore
```shell
make A=apps/task/sched-fairness-unequal LOG=info APP_FEATURES=... SMP=4 run
```

# RESULT
```
make A=apps/task/sched-fairness-unequal LOG=info APP_FEATURES=sched_mlfq SMP=4 run
...
part 0: TaskId(7) [0, 5000)
part 1: TaskId(8) [5000, 10000)
part 2: TaskId(9) [10000, 15000)
part 3: TaskId(10) [15000, 20000)
main task woken up! timeout=true
id 0, calc times = 65
id 1, calc times = 31
id 2, calc times = 21
id 3, calc times = 16
Parallel summation tests run OK!
```
# PROCESS

### 各算法参数设置

- cfs：默认情况下所有的 nice 值都是 0，此时期望是完全公平调度。
- sjf：移动平均系数默认设为 1/16，初值是 0
- mlfq：队列级数为 8，第 0 级队列分到 1 个时间片，过 100000 个时间片重置队列。同优先级抢占式调度。
- rr：MAX_TIME_SLICE=5

注：如果没有特殊说明，默认在**单核**模式下测试，时间单位为 **ms**。如果 cfs 设置了 nice 值，会额外说明。

### 设计思路

4 个进程，每个执行 50 组用时和进程号（0-index）加一成正比的任务，每组任务结束后主动 yield 一次，跑 2s，通过每个的计算次数，测试调度算法的公平性。

## COMPARATION
|      | 进程0运行次数 | 进程1运行次数 | 进程2运行次数 | 进程3运行次数 |
| ---- | ------------- | ------------- | ------------- | ------------- |
| rr   | 21            | 21            | 20            | 18            |
| fifo | 20            | 20            | 20            | 19            |
| cfs  | 49            | 24            | 15            | 12            |
| sjf  | 135           | 10            | 6             | 4             |
| mlfq | 76            | 37            | 9             | 2             |

## ANALYSIS
可以看出：

- cfs 各进程的运行次数和进程号加一大致成反比，具有良好的公平性
- sjf 公平性非常差，另外几个进程没有获得太多次运行的机会，全被进程 0 抢走了。这里另外几个进程也有运行次数的原因是，设置了预期时间初值是 0，且移动平均的系数较小（1/16），还是给了另外几个进程少量运行的机会。
- fifo 调度次数和任务负载无关。
- rr 这里由于每个循环的量较小（学习代码得知目前的实现是 10ms 一个时间片），所有的进程都落在 5 个时间片中，所以退化成了 fifo。
  - 进一步加大5倍循环量并设为5秒测试：分别是 17, 11, 8, 6，此时符合预期。

- mlfq 进程号大的较早就到了下一个优先级。两个进程工作量没差太多会落在同一个级别，一开始层内 fifo 调度，因为性能波动，工作量较大的那个有更大概率会波动到下一个优先级导致被较小的那个抢占，所以会出现不完全抢占现象。
