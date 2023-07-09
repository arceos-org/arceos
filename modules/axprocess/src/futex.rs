//! 实现与futex相关的系统调用

use core::time::Duration;

use alloc::collections::BTreeMap;
use axerrno::{AxError, AxResult};
use axhal::mem::VirtAddr;
use axtask::{monolithic_task::run_queue::WAIT_FOR_EXIT, AxTaskRef};
use spinlock::SpinNoIrq;

extern crate alloc;
use alloc::vec::Vec;

use crate::process::{current_process, current_task, sleep_now_task};

static FUTEX_WAIT_TASK: SpinNoIrq<BTreeMap<VirtAddr, Vec<AxTaskRef>>> =
    SpinNoIrq::new(BTreeMap::new());

/// 对 futex 的操作
pub enum Flags {
    /// 检查用户地址 uaddr 处的值。如果不是要求的值则等待 wake
    WAIT = 0,
    /// 唤醒最多 val 个在等待 uaddr 位置的线程。
    WAKE = 1,
    REQUEUE,
    UNSUPPORTED,
}

/// 传入的选项
pub struct FutexFlag(i32);

impl FutexFlag {
    pub fn new(val: i32) -> Self {
        Self(val)
    }

    pub fn operation(&self) -> Flags {
        match self.0 & 0x7f {
            0 => Flags::WAIT,
            1 => Flags::WAKE,
            3 => Flags::REQUEUE,
            _ => Flags::UNSUPPORTED,
        }
    }
}

pub struct FutexRobustList {
    pub head: usize,
    pub len: usize,
}

impl FutexRobustList {
    pub fn new(head: usize, len: usize) -> Self {
        Self { head, len }
    }
}

/// 退出的时候清空指针
pub fn clear_wait(process_id: u64) {
    let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
    // 清空所有所属进程为指定进程的线程
    futex_wait_task.iter_mut().for_each(|(_, tasks)| {
        let _ = tasks.extract_if(|task| task.get_process_id() == process_id);
    });
    // 如果一个共享变量不会被线程所使用了，那么直接把他移除
    let _ = futex_wait_task.extract_if(|_, tasks| tasks.is_empty());
}

pub fn futex(vaddr: VirtAddr, futex_op: i32, futex_val: u32, timeout: usize) -> AxResult<usize> {
    let flag = FutexFlag::new(futex_op);

    match flag.operation() {
        Flags::WAIT => {
            let current_task = current_task();
            let process = current_process();
            let inner = process.inner.lock();
            let mut memory_set = inner.memory_set.lock();
            if memory_set.manual_alloc_for_lazy(vaddr).is_ok() {
                let real_futex_val = unsafe { (vaddr.as_usize() as *const u32).read_volatile() };
                if real_futex_val != futex_val {
                    return Err(AxError::BadAddress);
                }
                let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
                let wait_list = if futex_wait_task.contains_key(&vaddr) {
                    futex_wait_task.get_mut(&vaddr).unwrap()
                } else {
                    futex_wait_task.insert(vaddr, Vec::new());
                    futex_wait_task.get_mut(&vaddr).unwrap()
                };
                wait_list.push(current_task.as_task_ref().clone());
                drop(futex_wait_task);
                drop(memory_set);
                drop(inner);
                sleep_now_task(Duration::from_nanos(timeout as u64));
                return Ok(0);
            } else {
                return Err(AxError::BadAddress);
            }
        }
        Flags::WAKE => {
            // 当前任务释放了锁，所以不需要再次释放
            let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
            if futex_wait_task.contains_key(&vaddr) {
                let wait_list = futex_wait_task.get_mut(&vaddr).unwrap();
                if let Some(task) = wait_list.pop() {
                    // 唤醒一个正在等待的任务
                    drop(futex_wait_task);
                    #[cfg(feature = "preempt")]
                    WAIT_FOR_EXIT.notify_task(true, &task);
                    #[cfg(not(feature = "preempt"))]
                    WAIT_FOR_EXIT.notify_task(false, &task);
                }
            }
            return Ok(futex_val as usize);
        }
        _ => {
            return Err(AxError::InvalidInput);
        }
    }
}
