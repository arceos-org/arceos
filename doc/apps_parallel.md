# INTRODUCTION

| App | Extra modules | Enabled features | Description |
| [parallel](apps/task/parallel/) | axalloc, axtask | alloc, paging, multitask, sched_fifo | Parallel computing test (to test synchronization & mutex) |

# RUN

```
make A=apps/task/parallel run 
...
part 0: TaskId(4) [0, 125000)
part 1: TaskId(5) [125000, 250000)
part 2: TaskId(6) [250000, 375000)
part 3: TaskId(7) [375000, 500000)
part 4: TaskId(8) [500000, 625000)
part 5: TaskId(9) [625000, 750000)
part 6: TaskId(10) [750000, 875000)
part 7: TaskId(11) [875000, 1000000)
part 8: TaskId(12) [1000000, 1125000)
part 9: TaskId(13) [1125000, 1250000)
part 10: TaskId(14) [1250000, 1375000)
part 11: TaskId(15) [1375000, 1500000)
part 12: TaskId(16) [1500000, 1625000)
part 13: TaskId(17) [1625000, 1750000)
part 14: TaskId(18) [1750000, 1875000)
part 15: TaskId(19) [1875000, 2000000)
part 15: TaskId(19) finished
main task woken up! timeout=true
sum = 61783189038
Parallel summation tests run OK!
```

# PROCESS

`main`使用`MAIN_WQ`睡眠 500ms，并检查`main`的唤醒是因为时间到（而非其他`task`的`notify()`）。

`main`调用`task::spawn`产生`NUM_TASKS`个`task`，分别进行计算。计算完毕后，使用一个`WaitQueue`（`static BARRIER_WQ`）以等待其他`task`的完成。
在全部`task`完成后，执行`BARRIER_WQ.notify_all(true)`，继续各`task`的执行。

`main`在生成`task`后，调用`MAIN_WQ.wait_timeout()`等待 600ms，随后检查`task`的计算结果。

# FUNCTIONS

## barrier

`BARRIER_COUNT += 1`，记录已经完成计算的`task`数量。

`BARRIER_WQ.wait_until()`，block 至所有`task`均完成计算。

`BARRIER_WQ.notify_all()`，唤醒`BARRIER_WQ`内的所有 task 继续执行。

# STEPS

## step1

[init](./init.md)

After executed all initial actions, then arceos calls `main` function in `parallel` app.

## step2

Calculate expected value from tasks.

```rust
let vec = Arc::new(
    (0..NUM_DATA)
        .map(|_| rand::rand_u32() as u64)
        .collect::<Vec<_>>(),
);
let expect: u64 = vec.iter().map(sqrt).sum();
```

## step3

Sleep `main` task in `MAIN_WQ` for 500ms. `main` **must** be timed out to wake up since there's no other task to `notify()` it.

```rust
let timeout = MAIN_WQ.wait_timeout(Duration::from_millis(500));
assert!(timeout);
```

## step4

`main` task spawn all `NUM_TASKS` tasks.

```rust
for i in 0..NUM_TASKS {
    let vec = vec.clone();
    task::spawn(move || {
        ...
    });
}
```

Each task will do the calculation, then call `barrier()`.

```rust
// task:
let left = i * (NUM_DATA / NUM_TASKS);
let right = (left + (NUM_DATA / NUM_TASKS)).min(NUM_DATA);
println!(
    "part {}: {:?} [{}, {})",
    i,
    task::current().id(),
    left,
    right
);

RESULTS.lock()[i] = vec[left..right].iter().map(sqrt).sum();

barrier();

println!("part {}: {:?} finished", i, task::current().id());
let n = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
if n == NUM_TASKS - 1 {
    MAIN_WQ.notify_one(true);
}

fn barrier() {
    static BARRIER_WQ: WaitQueue = WaitQueue::new();
    static BARRIER_COUNT: AtomicUsize = AtomicUsize::new(0);
    BARRIER_COUNT.fetch_add(1, Ordering::Relaxed);
    BARRIER_WQ.wait_until(|| BARRIER_COUNT.load(Ordering::Relaxed) == NUM_TASKS);
    BARRIER_WQ.notify_all(true);
}
```

`barrier()` will keep track of how many tasks have finished calculation in `BARRIER_COUNT`.

Task will sleep in `BARRIER_WQ` until all tasks have finished. Then, the first awake task will `notify_all()` tasks to wake up.

Task will print some info, add 1 to `FINISHED_TASKS`. The last task (`n == NUM_TASKS - 1`) will notify the `main` task to wake up.

## step5

`main` will sleep 600ms in `MAIN_WQ` after spawning all the tasks. Once awake, `main` will check the actual calculation results.

```rust
let timeout = MAIN_WQ.wait_timeout(Duration::from_millis(600));
println!("main task woken up! timeout={}", timeout);

let actual = RESULTS.lock().iter().sum();
println!("sum = {}", actual);
assert_eq!(expect, actual);

println!("Parallel summation tests run OK!");
```
