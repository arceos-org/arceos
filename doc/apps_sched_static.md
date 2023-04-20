# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [sched-static](../apps/task/sched-static/) | axalloc, axtask | alloc, paging, multitask, sched_rms | schedule test short payload & yield|

# RUN
```shell
make A=apps/task/sched-static LOG=info run

## Using multicore
```shell
make A=apps/task/sched-static LOG=info SMP=3 run
```

# RESULT
```
make A=apps/task/sched-realtime LOG=info APP_FEATURES=sched_mlfq SMP=4 run
...
part 0: TaskId(6) [0, 1000)
runtime = 1, sleeptime = 2
part 1: TaskId(5) [0, 1000)
runtime = 2, sleeptime = 3
part 2: TaskId(4) [0, 1000)
runtime = 1, sleeptime = 5
runtime = 1, sleeptime = 2
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 1, sleeptime = 2
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 1, sleeptime = 2
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 1, sleeptime = 2
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 1, sleeptime = 2
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 2, sleeptime = 3
runtime = 1, sleeptime = 2
runtime = 1, sleeptime = 5
runtime = 1, sleeptime = 2
runtime = 2, sleeptime = 3
... 
```
# PROCESS

### 设计思路：

有三个任务 {τ1, τ2, τ3}

任务属性如下：

任务 τ1：执行时间 C1 = 1, 周期 T1 = 3
任务 τ2：执行时间 C2 = 2, 周期 T2 = 5
任务 τ3：执行时间 C3 = 1, 周期 T3 = 6

这里使用一定时间的循环来模拟这个执行时间。