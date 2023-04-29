# INTRODUCTION

| App | Extra modules | Enabled features | Description |
|-|-|-|-|
| [priority](../apps/task/priority/) | axalloc, axtask | alloc, paging, multitask, sched_fifo, sched_rr, sched_cfs | test priority according to cfs::nice|

# RUN
```shell
make A=apps/task/priority ARCH=riscv64 SMP=1 APP_FEATURES=sched_cfs run LOG=info
```
Other choises of APP_FEATURES: sched_fifo, sched_rr
## Using multicore
```shell
make A=apps/task/sched-realtime ARCH=riscv64 SMP=4 APP_FEATURES=sched_cfs run LOG=info 
```
Other choises of APP_FEATURES: sched_fifo, sched_rr

# RESULT
```
make A=apps/task/priority ARCH=riscv64 SMP=1 APP_FEATURES=sched_cfs run LOG=info
...
part 0: TaskId(4) [0, 40)
part 1: TaskId(5) [0, 40)
part 2: TaskId(6) [0, 40)
part 3: TaskId(7) [0, 40)
part 4: TaskId(8) [0, 4)
part 3: TaskId(7) finished
part 4: TaskId(8) finished
part 2: TaskId(6) finished
part 1: TaskId(5) finished
part 0: TaskId(4) finished
sum = 3318102132
leave time:
task 0 = 614ms
task 1 = 479ms
task 2 = 374ms
task 3 = 166ms
task 4 = 371ms
Priority tests run OK!
[  1.274073 0:2 axhal::platform::qemu_virt_riscv::misc:3] Shutting down...
```