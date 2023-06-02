#### 什么时候会发生抢占

- 1 关抢占（且当前的抢占计数器为0）再开抢占
- 2 unblock

#### 需要被抢占和真的被抢占的区别

- need_resched 设为 True 的时机
- resched 被调用的时机

#### 一个时钟中断来的时候会如何进行调度

on_timer_tick(这个时候关抢占) -> scheduler_timer_tick -> task_tick -> set_preempt_pending -> need_resched True -> on_timer_tick退出（开抢占）-> 触发 current_check_preempt_pending（因为 need_resched = True) ->  resched（这里在修复bug之后还是需要关抢占）

如果 preempt=true -> resched_inner -> switch_to -> 切到下一个线程，此时抢占信息是只关于下一个线程的 -> ... -> 切回到这个线程到 ```(*prev_ctx_ptr).switch_to(&*next_ctx_ptr)``` 的 pc+4 执行 -> resched 结束（这里 关抢占的上下文结束，会开抢占）-> 再触发一次 current_check_preempt_pending -> (由于在 switch_to 中设置了 need_resched = false) 不会继续 resched，结束。



on_timer_tick 的开关中断：axruntime/trap/... 前后会有一个开关中断。

#### 抢占和中断的区别

- 中断是硬件产生的，比如时钟中断
- 抢占：软件中因为其他优先级更高而强行切换线程。

#### 目前 bug 的解决

- 给每个 RUN_QUEUE 增加 NoPreempt 上下文，这样每个进程能完整地执行完整个 resched 过程而不会被抢。

