//! 实现与futex相关的系统调用
use alloc::collections::{BTreeMap, VecDeque};
use axhal::mem::VirtAddr;
use axtask::AxTaskRef;
use spinlock::SpinNoIrq;

extern crate alloc;

/// vec中的元素分别是任务指针以及对应存储时的futex变量的值
pub static FUTEX_WAIT_TASK: SpinNoIrq<BTreeMap<VirtAddr, VecDeque<(AxTaskRef, u32)>>> =
    SpinNoIrq::new(BTreeMap::new());

pub struct FutexRobustList {
    pub head: usize,
    pub len: usize,
}

impl Default for FutexRobustList {
    fn default() -> Self {
        Self { head: 0, len: 0 }
    }
}
impl FutexRobustList {
    pub fn new(head: usize, len: usize) -> Self {
        Self { head, len }
    }
}

/// 退出的时候清空指针
///
/// 若当前线程是主线程，代表进程退出，此时传入的id是进程id，要清除所有进程下的线程
///
/// 否则传入的id是线程id
pub fn clear_wait(id: u64, leader: bool) {
    let mut futex_wait_task = FUTEX_WAIT_TASK.lock();

    if leader {
        // 清空所有所属进程为指定进程的线程
        futex_wait_task.iter_mut().for_each(|(_, tasks)| {
            // tasks.drain_filter(|task| task.get_process_id() == id);
            tasks.retain(|(task, _)| task.get_process_id() != id);
        });
    } else {
        futex_wait_task.iter_mut().for_each(|(_, tasks)| {
            // tasks.drain_filter(|task| task.id().as_u64() == id);
            tasks.retain(|(task, _)| task.id().as_u64() != id)
        });
    }

    // 如果一个共享变量不会被线程所使用了，那么直接把他移除
    // info!("clean pre keys: {:?}", futex_wait_task.keys());
    futex_wait_task.drain_filter(|_, tasks| tasks.is_empty());
}
