#![cfg(test)]

#[test]
fn test_sched_fifo() {
    use core::sync::atomic::{AtomicUsize, Ordering};

    const NUM_TASKS: usize = 10;
    static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

    crate::init_scheduler();
    for i in 0..NUM_TASKS {
        crate::spawn(move || {
            // println!("Hello, task {}! id = {:?}", i, crate::current().id());
            // TODO: context-switch SIMD registers to use print without crash
            crate::yield_now();
            let order = FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
            assert!(order == i); // FIFO scheduler
        });
    }
    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        crate::yield_now();
    }
}
