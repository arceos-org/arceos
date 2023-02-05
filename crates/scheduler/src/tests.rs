macro_rules! def_test_sched {
    ($name: ident, $scheduler: ident, $task: ident) => {
        mod $name {
            use crate::*;
            use alloc::sync::Arc;

            #[test]
            fn test_sched() {
                const NUM_TASKS: usize = 11;

                let mut schedulder = $scheduler::new();
                for i in 0..NUM_TASKS {
                    schedulder.add_task(Arc::new($task::new(i)));
                }

                let mut current = schedulder.pick_next_task().unwrap().clone();
                for i in 0..NUM_TASKS * 10 - 1 {
                    assert_eq!(*current.inner(), i % NUM_TASKS);
                    schedulder.yield_task(&current);
                    current = schedulder.pick_next_task().unwrap().clone();
                }

                let mut prev_id = *current.inner();
                loop {
                    schedulder.remove_task(&current);
                    if let Some(cur) = schedulder.pick_next_task().cloned() {
                        assert_eq!(*cur.inner(), (prev_id + 1) % NUM_TASKS);
                        prev_id = *cur.inner();
                        current = cur;
                    } else {
                        break;
                    }
                }
            }

            #[test]
            fn bench_yield() {
                const NUM_TASKS: u32 = 1_000_000;
                const COUNT: u32 = NUM_TASKS * 3;

                let mut schedulder = $scheduler::new();
                for i in 0..NUM_TASKS {
                    schedulder.add_task(Arc::new($task::new(i)));
                }

                let mut current = schedulder.pick_next_task().unwrap().clone();

                let t0 = std::time::Instant::now();
                for _ in 0..COUNT {
                    schedulder.yield_task(&current);
                    current = schedulder.pick_next_task().unwrap().clone();
                }
                let t1 = std::time::Instant::now();
                println!(
                    "{}: task yield speed: {:?}",
                    stringify!($scheduler),
                    (t1 - t0) / COUNT
                );
            }
        }
    };
}

def_test_sched!(fifo, FifoScheduler, FifoTask);
def_test_sched!(rr, RRScheduler, RRTask);
