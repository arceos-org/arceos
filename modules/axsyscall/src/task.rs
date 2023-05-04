use axprocess::{
    flags::{CloneFlags, WaitStatus},
    mem::memory_set::get_app_data,
    process::{wait_pid, PID2PC},
};
use axtask::current;
extern crate alloc;
use alloc::{sync::Arc, vec::Vec};
use log::info;

use crate::flags::WaitFlags;
/// 处理与任务（线程）有关的系统调用

pub fn syscall_exit(exit_code: i32) -> isize {
    axlog::info!("Syscall to exit!");
    axprocess::process::exit(exit_code)
}

pub fn syscall_exec(path: *const u8, mut args: *const usize) -> isize {
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    let inner = curr_process.inner.lock();
    let path = inner.memory_set.lock().translate_str(path);
    axlog::info!("path: {}", path);
    axlog::info!("Syscall to exec {}", path);
    let mut args_vec = Vec::new();
    loop {
        let args_str_ptr = unsafe { *args };
        if args_str_ptr == 0 {
            break;
        }
        args_vec.push(
            inner
                .memory_set
                .lock()
                .translate_str(args_str_ptr as *const u8),
        );
        unsafe {
            args = args.add(1);
        }
    }
    drop(inner);
    let elf_data = get_app_data(&path);
    let argc = args_vec.len();
    curr_process.exec(elf_data, args_vec);
    argc as isize
}

pub fn syscall_clone(
    flags: usize,
    user_stack: usize,
    ptid: usize,
    tls: usize,
    ctid: usize,
) -> isize {
    let clone_flags = CloneFlags::from_bits(flags as u32).unwrap();
    let stack = if user_stack == 0 {
        None
    } else {
        Some(user_stack)
    };
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    let new_task_id = curr_process.clone_task(clone_flags, stack, ptid, tls, ctid);
    new_task_id as isize
}

/// 当前不涉及多核情况
pub fn syscall_getpid() -> isize {
    let curr = current();
    let pid = curr.get_process_id();
    pid as isize
}

pub fn syscall_getppid() -> isize {
    let curr = current();
    let pid2pc_inner = PID2PC.lock();
    let curr_process = Arc::clone(&pid2pc_inner.get(&curr.get_process_id()).unwrap());
    drop(pid2pc_inner);
    let inner = curr_process.inner.lock();
    let parent_id = inner.parent;
    drop(inner);
    parent_id as isize
}

/// 等待子进程完成任务，若子进程没有完成，则自身yield
/// 当前仅支持WNOHANG选项，即若未完成时则不予等待，直接返回0
pub fn syscall_wait4(pid: isize, exit_code_ptr: *mut i32, option: WaitFlags) -> isize {
    loop {
        let answer = wait_pid(pid, exit_code_ptr);
        match answer {
            Ok(pid) => {
                info!("sub task finish: {}", pid);
                return pid as isize;
            }
            Err(status) => {
                match status {
                    WaitStatus::NotExist => {
                        info!("Not exist!");
                        return -1;
                    }
                    WaitStatus::Running => {
                        info!("Is running!");
                        if option.contains(WaitFlags::WNOHANG) {
                            // 不予等待，直接返回0
                            return 0;
                        } else {
                            // 执行yield操作，切换任务
                            info!("wait4: yield_now");
                            axtask::yield_now();
                        }
                    }
                    _ => {
                        panic!("Shouldn't reach here!");
                    }
                }
            }
        };
    }
}
