use syscall_utils::deal_result;

#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    #[cfg(feature = "futex")]
    syscall_task::check_dead_wait();
    let ans = loop {
        #[cfg(feature = "syscall_net")]
        {
            if let Ok(net_syscall_id) = syscall_net::NetSyscallId::try_from(syscall_id) {
                break syscall_net::net_syscall(net_syscall_id, args);
            }
        }

        #[cfg(feature = "syscall_mem")]
        {
            if let Ok(mem_syscall_id) = syscall_mem::MemSyscallId::try_from(syscall_id) {
                break syscall_mem::mem_syscall(mem_syscall_id, args);
            }
        }

        #[cfg(feature = "syscall_fs")]
        {
            if let Ok(fs_syscall_id) = syscall_fs::FsSyscallId::try_from(syscall_id) {
                break syscall_fs::fs_syscall(fs_syscall_id, args);
            }
        }

        // #[cfg(feature = "syscall_task")]

        {
            if let Ok(task_syscall_id) = syscall_task::TaskSyscallId::try_from(syscall_id) {
                break syscall_task::task_syscall(task_syscall_id, args);
            }
        }

        panic!("unknown syscall id: {}", syscall_id);
    };

    let ans = deal_result(ans);
    axlog::info!("syscall: {} -> {}", syscall_id, ans);
    ans
}
