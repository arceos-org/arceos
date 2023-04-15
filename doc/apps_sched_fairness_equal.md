# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [sched_fairness_equal](../apps/task/sched_fairness_equal/) | axalloc, axtask | alloc, paging, multitask, sched_rr, sched_fifo, sched_mlfq, sched_sjf, sched_cfs | schedule test equal payload|

# RUN
```shell
make A=apps/task/sched-fairness-equal LOG=info APP_FEATURES=sched_cfs run

Other choises of APP_FEATURES: sched_rr, sched_fifo, sched_mlfq, sched_sjf
```

## Using multicore
```shell
make A=apps/task/sched-fairness-equal LOG=info APP_FEATURES=... SMP=4 run
```

# RESULT
```
make A=apps/task/sched-fairness-equal LOG=info APP_FEATURES=sched_mlfq SMP=4 run
...
part 0: TaskId(7) [0, 25)
part 1: TaskId(8) [25, 50)
part 2: TaskId(9) [50, 75)
part 3: TaskId(10) [75, 100)
part 3: TaskId(10) finished
part 2: TaskId(9) finished
part 1: TaskId(8) finished
part 0: TaskId(7) finished
main task woken up! timeout=false
leave time id 0 = 424ms
leave time id 1 = 410ms
leave time id 2 = 398ms
leave time id 3 = 434ms
maximum leave time = 434ms
sum = 1249999004807739
minimum leave time = 398ms
Parallel summation tests run OK!
[  2.637205 2:2 axhal::platform::qemu_virt_riscv::misc:2] Shutting down...
```
# PROCESS

### 各算法参数设置

- cfs：默认情况下所有的 nice 值都是 0，此时期望是完全公平调度。
- sjf：移动平均系数默认设为 1/16，初值是 0
- mlfq：队列级数为 8，第 0 级队列分到 1 个时间片，过 100000 个时间片重置队列。同优先级抢占式调度。
- rr：MAX_TIME_SLICE=5


注：如果没有特殊说明，默认在**单核**模式下测试，时间单位为 **ms**。如果 cfs 设置了 nice 值，会额外说明。

### 设计思路

4 个进程，每个执行 **100** 组 **用时相同的任务**，每组任务结束后主动 yield 一次，初步测试调度算法的公平性

- 用时相同：每组任务需要执行一个长度为 1000000 的循环。

## COMPARATION
|      | 进程0结束时间 | 进程1结束时间 | 进程2结束时间 | 进程3结束时间 |
| ---- | ------------- | ------------- | ------------- | ------------- |
| rr   | 1291          | 1277          | 1265          | 1253          |
| fifo | 1302          | 1282          | 1265          | 1249          |
| cfs  | 1281          | 1267          | 1241          | 1243          |
| sjf  | 1262          | 1263          | 1249          | 1235          |
| mlfq | 1226          | 1150          | 1237          | 1224          |
