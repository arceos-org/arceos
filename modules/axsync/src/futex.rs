use core::sync::atomic::{AtomicU32, Ordering};

use alloc::{collections::BTreeMap, sync::Arc};
use axtask::WaitQueue;
use lazy_init::LazyInit;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

extern crate alloc;

struct FutexPool(SpinNoIrq<BTreeMap<PhysAddr, Arc<WaitQueue>>>);

impl FutexPool {
    pub fn current_wait(&self, paddr: PhysAddr, val: u32) -> bool {
        if !paddr.is_aligned(4usize) {
            panic!("Align is invalid!");
        }

        let data: &'static AtomicU32 = unsafe { &*(paddr.as_usize() as *const u32).cast() };

        if data.load(Ordering::Acquire) != val {
            return false;
        }
        if let Some(queue) = self.0.lock().get(&paddr).map(Arc::clone) {
            queue.wait();
        } else {
            let queue = Arc::new(WaitQueue::new());
            self.0.lock().insert(paddr, queue.clone()).unwrap();
            queue.wait();
        }
        true
    }
    pub fn current_wake(&self, paddr: PhysAddr, val: u32) -> u32 {
        for i in 0..val {
            if let Some(queue) = self.0.lock().get(&paddr).map(Arc::clone) {
                queue.notify_one(true);
                if queue.len() == 0 {
                    self.0.lock().remove(&paddr);
                }
            } else {
                return i;
            }
        }

        val
    }
}

static FUTEX_GLOBAL_POOL: LazyInit<FutexPool> = LazyInit::new();
pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;
pub fn init() {
    FUTEX_GLOBAL_POOL.init_by(FutexPool(SpinNoIrq::new(BTreeMap::new())));
}
pub fn futex_call(paddr: usize, op: usize, val: u32) -> isize {
    match op {
        FUTEX_WAIT => {
            if FUTEX_GLOBAL_POOL.current_wait(paddr.into(), val) {
                0
            } else {
                -2
            }
        }
        FUTEX_WAKE => FUTEX_GLOBAL_POOL.current_wake(paddr.into(), val) as isize,
        _ => -1,
    }
}
