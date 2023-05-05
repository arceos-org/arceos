use core::sync::atomic::{AtomicU32, Ordering};

use axtask::WaitQueue;
use memory_addr::PhysAddr;

/// kernel structure for `futex`.
pub struct Futex {
    data: &'static AtomicU32,
    wq: WaitQueue,
}

impl Futex {
    pub fn wait(&mut self, val: u32) {
        if self.data.load(Ordering::Acquire) != val {
            self.wq.wait_until(|| {
                self.data.load(Ordering::Relaxed) == val
            });
        }
    }

    pub fn wake(&mut self, _val: u32) {
        self.wq.notify_one(true);
    }

    pub fn new(paddr: PhysAddr) -> Futex {
        if !paddr.is_aligned(4usize) {
            panic!("addr is not properly aligned!");                
        }
        Futex {
            data: unsafe {
                & *(paddr.as_usize() as *const AtomicU32)
            },
            wq: WaitQueue::new()
        }
    }
}

pub fn futex_call(_paddr: usize, _op: usize, _val: u32) {
    unimplemented!();
}
