//! 支持与任务调度相关的 syscall
extern crate alloc;
use alloc::sync::Arc;
use axconfig::SMP;
use axhal::mem::VirtAddr;
use axprocess::{current_process, current_task, futex::FutexRobustList, PID2PC, TID2TASK};

#[cfg(feature = "signal")]
use axtask::{SchedPolicy, SchedStatus};
use syscall_utils::{RobustList, SchedParam, SyscallError, SyscallResult};

/// 内核只发挥存储的作用
/// 但要保证head对应的地址已经被分配
pub fn syscall_set_robust_list(head: usize, len: usize) -> SyscallResult {
    let process = current_process();
    if len != core::mem::size_of::<RobustList>() {
        return Err(SyscallError::EINVAL);
    }
    let curr_id = current_task().id().as_u64();
    if process.manual_alloc_for_lazy(head.into()).is_ok() {
        let mut robust_list = process.robust_list.lock();
        if robust_list.contains_key(&curr_id) {
            let list = robust_list.get_mut(&curr_id).unwrap();
            list.head = head;
            list.len = len;
        } else {
            robust_list.insert(curr_id, FutexRobustList::new(head, len));
        }
        Ok(0)
    } else {
        Err(SyscallError::EINVAL)
    }
}

/// 取出对应线程的robust list
pub fn syscall_get_robust_list(pid: i32, head: *mut usize, len: *mut usize) -> SyscallResult {
    if pid == 0 {
        let process = current_process();
        let curr_id = current_task().id().as_u64();
        if process
            .manual_alloc_for_lazy((head as usize).into())
            .is_ok()
        {
            let robust_list = process.robust_list.lock();
            if robust_list.contains_key(&curr_id) {
                let list = robust_list.get(&curr_id).unwrap();
                unsafe {
                    *head = list.head;
                    *len = list.len;
                }
            } else {
                return Err(SyscallError::EPERM);
            }
            return Ok(0);
        } else {
            return Err(SyscallError::EPERM);
        }
    }
    Err(SyscallError::EPERM)
}

/// 获取对应任务的CPU适配集
///
/// 若pid是进程ID，则获取对应的进程的主线程的信息
///
/// 若pid是线程ID，则获取对应线程信息
///
/// 若pid为0，则获取当前运行任务的信息
///
/// mask为即将写入的cpu set的地址指针
pub fn syscall_sched_getaffinity(
    pid: usize,
    cpu_set_size: usize,
    mask: *mut usize,
) -> SyscallResult {
    // let task: LazyInit<AxTaskRef> = LazyInit::new();
    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let process = current_process();
    if process
        .manual_alloc_for_lazy(VirtAddr::from(mask as usize))
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }
    let cpu_set = task.get_cpu_set();
    let mut prev_mask = unsafe { *mask };
    let len = SMP.min(cpu_set_size * 4);
    prev_mask &= !((1 << len) - 1);
    prev_mask &= cpu_set & ((1 << len) - 1);
    unsafe {
        *mask = prev_mask;
    }
    // 返回成功填充的缓冲区的长度
    Ok(SMP as isize)
}

#[allow(unused)]
pub fn syscall_sched_setaffinity(
    pid: usize,
    cpu_set_size: usize,
    mask: *const usize,
) -> SyscallResult {
    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let process = current_process();
    if process
        .manual_alloc_for_lazy(VirtAddr::from(mask as usize))
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }

    let mask = unsafe { *mask };

    task.set_cpu_set(mask, cpu_set_size);

    Ok(0)
}

pub fn syscall_sched_setscheduler(
    pid: usize,
    policy: usize,
    param: *const SchedParam,
) -> SyscallResult {
    if (pid as isize) < 0 || param.is_null() {
        return Err(SyscallError::EINVAL);
    }

    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let process = current_process();
    if process
        .manual_alloc_for_lazy(VirtAddr::from(param as usize))
        .is_err()
    {
        return Err(SyscallError::EFAULT);
    }

    let param = unsafe { *param };
    let policy = SchedPolicy::from(policy);
    if policy == SchedPolicy::SCHED_UNKNOWN {
        return Err(SyscallError::EINVAL);
    }
    if policy == SchedPolicy::SCHED_OTHER
        || policy == SchedPolicy::SCHED_BATCH
        || policy == SchedPolicy::SCHED_IDLE
    {
        if param.sched_priority != 0 {
            return Err(SyscallError::EINVAL);
        }
    } else {
        if param.sched_priority < 1 || param.sched_priority > 99 {
            return Err(SyscallError::EINVAL);
        }
    }

    task.set_sched_status(SchedStatus {
        policy,
        priority: param.sched_priority,
    });

    Ok(0)
}

pub fn syscall_sched_getscheduler(pid: usize) -> SyscallResult {
    if (pid as isize) < 0 {
        return Err(SyscallError::EINVAL);
    }

    let tid2task = TID2TASK.lock();
    let pid2task = PID2PC.lock();
    let pid = pid as u64;
    let task = if tid2task.contains_key(&pid) {
        Arc::clone(&tid2task.get(&pid).unwrap())
    } else if pid2task.contains_key(&pid) {
        let process = pid2task.get(&pid).unwrap();

        process
            .tasks
            .lock()
            .iter()
            .find(|task| task.is_leader())
            .map(|task| Arc::clone(task))
            .unwrap()
    } else if pid == 0 {
        Arc::clone(current_task().as_task_ref())
    } else {
        // 找不到对应任务
        return Err(SyscallError::ESRCH);
    };

    drop(pid2task);
    drop(tid2task);

    let policy: isize = task.get_sched_status().policy.into();
    Ok(policy)
}
