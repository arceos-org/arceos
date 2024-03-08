extern crate alloc;
#[allow(unused)]
use alloc::format;
use alloc::string::String;
use syscall_utils::deal_result;
use syscall_utils::SyscallResult;
pub fn syscall_name(syscall_id: usize) -> String {
    #[allow(unused_mut)]
    let mut name: Option<String> = None;
    #[cfg(feature = "syscall_net")]
    if let Ok(syscall_num) = syscall_net::NetSyscallId::try_from(syscall_id) {
        name = Some(format!("{:?}", syscall_num));
    }
    #[cfg(feature = "syscall_mem")]
    if let Ok(syscall_num) = syscall_mem::MemSyscallId::try_from(syscall_id) {
        name = Some(format!("{:?}", syscall_num));
    }
    #[cfg(feature = "syscall_fs")]
    if let Ok(syscall_num) = syscall_fs::FsSyscallId::try_from(syscall_id) {
        name = Some(format!("{:?}", syscall_num));
    }
    #[cfg(feature = "syscall_task")]
    if let Ok(syscall_num) = syscall_task::TaskSyscallId::try_from(syscall_id) {
        name = Some(format!("{:?}", syscall_num));
    }

    if name.is_none() {
        panic!("unknown syscall id: {}", syscall_id);
    }

    name.unwrap()
}

#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    #[cfg(feature = "futex")]
    syscall_task::check_dead_wait();
    #[allow(unused_mut)]
    let mut ans: Option<SyscallResult> = None;
    #[cfg(feature = "syscall_net")]
    {
        if let Ok(net_syscall_id) = syscall_net::NetSyscallId::try_from(syscall_id) {
            axlog::info!(
                "[syscall] id = {:#?}, args = {:?}, entry",
                net_syscall_id,
                args
            );
            ans = Some(syscall_net::net_syscall(net_syscall_id, args));
        }
    }

    #[cfg(feature = "syscall_mem")]
    {
        if let Ok(mem_syscall_id) = syscall_mem::MemSyscallId::try_from(syscall_id) {
            axlog::info!(
                "[syscall] id = {:#?}, args = {:?}, entry",
                mem_syscall_id,
                args
            );
            ans = Some(syscall_mem::mem_syscall(mem_syscall_id, args));
        }
    }

    #[cfg(feature = "syscall_fs")]
    {
        if let Ok(fs_syscall_id) = syscall_fs::FsSyscallId::try_from(syscall_id) {
            axlog::info!(
                "[syscall] id = {:#?}, args = {:?}, entry",
                fs_syscall_id,
                args
            );
            ans = Some(syscall_fs::fs_syscall(fs_syscall_id, args));
        }
    }

    #[cfg(feature = "syscall_task")]
    {
        if let Ok(task_syscall_id) = syscall_task::TaskSyscallId::try_from(syscall_id) {
            axlog::info!(
                "[syscall] id = {:#?}, args = {:?}, entry",
                task_syscall_id,
                args
            );
            ans = Some(syscall_task::task_syscall(task_syscall_id, args));
        }
    }
    if ans.is_none() {
        panic!("unknown syscall id: {}", syscall_id);
    };

    let ans = ans.unwrap();

    let ans = deal_result(ans);
    axlog::info!(
        "syscall: {} -> {} args: {:?}",
        syscall_name(syscall_id),
        ans,
        args
    );
    ans
}
