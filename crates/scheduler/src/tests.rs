macro_rules! def_test_sched {
    ($name: ident, $scheduler: ty, $task: ty) => {
        mod $name {
            use crate::*;
            use alloc::sync::Arc;
            use core::time::Duration;

            #[test]
            fn test_sched() {
                //axlog::init();
                //axlog::set_max_level("info");
                const NUM_TASKS: usize = 11;

                let mut scheduler = <$scheduler>::new();
                for i in 0..NUM_TASKS {
                    scheduler.add_task(Arc::new(<$task>::new(i, 0)));
                }

                for i in 0..NUM_TASKS * 10 - 1 {
                    let next = scheduler.pick_next_task().unwrap();
                    assert_eq!(*next.inner(), i % NUM_TASKS);
                    // 理论上需要过一个时间片，否则顺序不对
                    std::thread::sleep(Duration::from_micros(100 + (i * 500) as u64));
                    scheduler.task_tick(&next);
                    scheduler.put_prev_task(next, false);
                }

                let mut n = 0;
                while scheduler.pick_next_task().is_some() {
                    n += 1;
                }
                assert_eq!(n, NUM_TASKS);
            }

            #[test]
            fn bench_yield() {
                const NUM_TASKS: usize = 1_000_000;
                const COUNT: usize = NUM_TASKS * 3;

                let mut scheduler = <$scheduler>::new();
                for i in 0..NUM_TASKS {
                    scheduler.add_task(Arc::new(<$task>::new(i, 0)));
                }

                let t0 = std::time::Instant::now();
                for _ in 0..COUNT {
                    let next = scheduler.pick_next_task().unwrap();
                    scheduler.put_prev_task(next, false);
                }
                let t1 = std::time::Instant::now();
                println!(
                    "  {}: task yield speed: {:?}/task",
                    stringify!($scheduler),
                    (t1 - t0) / (COUNT as u32)
                );
            }

            #[test]
            fn bench_remove() {
                const NUM_TASKS: usize = 10_000;

                let mut scheduler = <$scheduler>::new();
                let mut tasks = Vec::new();
                for i in 0..NUM_TASKS {
                    let t = Arc::new(<$task>::new(i, 0));
                    tasks.push(t.clone());
                    scheduler.add_task(t);
                }

                let t0 = std::time::Instant::now();
                for i in (0..NUM_TASKS).rev() {
                    let t = scheduler.remove_task(&tasks[i]).unwrap();
                    assert_eq!(*t.inner(), i);
                }
                let t1 = std::time::Instant::now();
                println!(
                    "  {}: task remove speed: {:?}/task",
                    stringify!($scheduler),
                    (t1 - t0) / (NUM_TASKS as u32)
                );
            }
            #[test]
            fn bench_nice_fairness() {
                // 最基本的公平性测试。为了方便，nice 为各个值的任务的任务恰好一个。观察会被取出来几次。
                // 所有的测试都能 pass，通过输出查看每个 nice 的公平性。
                // TODO：现在的输出暂时有点乱
                const COUNT: usize = 3_000_000;
                const MNnice: isize = -3;
                const MXnice: isize = 3;
                let mut scheduler = <$scheduler>::new();
                
                for i in MNnice .. MXnice {
                    scheduler.add_task(Arc::new(<$task>::new(i as usize, i)));
                }
                let mut cnt = [0 as usize; (MXnice - MNnice + 1) as usize];
                for _ in 0..COUNT {
                    let next = scheduler.pick_next_task().unwrap();
                    cnt[(*next.inner() as isize - MNnice) as usize] += 1;
                    scheduler.task_tick(&next);
                    scheduler.put_prev_task(next, false);
                }
                for i in MNnice .. MXnice {
                    println!(
                        "{}: ratio for nice {}: {}",
                        stringify!($scheduler),
                        i,
                        (cnt[(i - MNnice) as usize] as f32) / (COUNT as f32)
                    )
                }
            }
            #[test]
            fn bench_real_time() {
                // TODO: 测试实时性
            }
        }
    };
}

def_test_sched!(fifo, FifoScheduler::<usize>, FifoTask::<usize>);
def_test_sched!(rr, RRScheduler::<usize, 5>, RRTask::<usize, 5>);
def_test_sched!(cfs, CFScheduler::<usize>, CFTask::<usize>);
def_test_sched!(sjf, SJFScheduler::<usize, 1, 4>, SJFTask::<usize, 1, 4>); // alpha=1/16
def_test_sched!(mlfq, MLFQScheduler::<usize, 8, 1, 100_000>, MLFQTask::<usize, 8, 1, 100_000>); // alpha=1/16
