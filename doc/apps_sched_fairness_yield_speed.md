# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [sched-yield-speed](../apps/task/sched-yield-speed/) | axalloc, axtask | alloc, paging, multitask, sched_rr, sched_fifo, sched_mlfq, sched_sjf, sched_cfs | schedule test short payload & yield|

# RUN
```shell
make A=apps/task/sched-yield-speed LOG=info APP_FEATURES=sched_cfs run

Other choises of APP_FEATURES: sched_rr, sched_fifo, sched_mlfq, sched_sjf
```

## Using multicore
```shell
make A=apps/task/sched-yield-speed LOG=info APP_FEATURES=... SMP=4 run
```

# RESULT
```
make A=apps/task/sched-yield-speed LOG=info APP_FEATURES=sched_mlfq SMP=4 run
...
part 0: TaskId(7) [0, 100000)
part 1: TaskId(8) [100000, 200000)
part 2: TaskId(9) [200000, 300000)
part 3: TaskId(10) [300000, 400000)
part 1: TaskId(8) finished
part 0: TaskId(7) finished
part 2: TaskId(9) finished
part 3: TaskId(10) finished
main task woken up! timeout=false
leave time id 0 = 247ms
leave time id 1 = 265ms
leave time id 2 = 261ms
leave time id 3 = 265ms
maximum leave time = 265ms
sum = 495016780577
minimum leave time = 247ms
Parallel summation tests run OK!
[  1.339823 1:2 axhal::platform::qemu_virt_riscv::misc:2] Shutting down...
```
# PROCESS

### 各算法参数设置

- cfs：默认情况下所有的 nice 值都是 0，此时期望是完全公平调度。
- sjf：移动平均系数默认设为 1/16，初值是 0
- mlfq：队列级数为 8，第 0 级队列分到 1 个时间片，过 100000 个时间片重置队列。同优先级抢占式调度。
- rr：MAX_TIME_SLICE=5

注：如果没有特殊说明，默认在**单核**模式下测试，时间单位为 **ms**。如果 cfs 设置了 nice 值，会额外说明。

### 设计思路：
4 个进程，每个执行 100000 组用时固定的任务（100次循环），每组任务结束后主动 yield 一次。

这个测例可以测试调度算法的基础速度，以及一定程度上反映公平性

## COMPARATION 
|      | 进程0运行时间 | 进程1运行时间 | 进程2运行时间 | 进程3运行时间 |
| ---- | ------------- | ------------- | ------------- | ------------- |
| fifo | 281           | 280           | 280           | 279           |
| rr   | 425           | 424           | 423           | 423           |
| cfs  | 448           | 453           | 436           | 434           |
| sjf  | 466           | 245           | 120           | 371           |
| mlfq | 290           | 301           | 253           | 297           |


## ANALYSIS

- 速度：fifo > mlfq >> rr > cfs ~ sjf。fifo 的计算最简单，调度速度最快。mlfq 当任务很短的时候几乎没啥计算。cfs 和 sfj 都需要优先队列，调度速度略慢。
- 公平性：这里 fifo, rr, cfs 表现得较为公平，sjf 由于波动会导致这些任务跑得不一样快，结果一开始波动到最快的任务抢占了队列，且由于移动平均权重设得较小（当前占 1/16），导致这个不公平进行了累积，导致了最后一个一个进程跑。mlfq 有一点不公平，原因是性能波动导致的有些进程早点进入下一级优先级，被抢占了一段时间，所以最后还需要一段时间跑。
