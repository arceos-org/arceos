use self::{fs::syscall_write, task::syscall_exit};

/// 处理系统调用
/// 负责系统调用的分发与处理
mod fs;
mod task;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;

#[cfg(feature = "user")]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => syscall_exit(),
        _ => {
            panic!("Invalid Syscall Id: {}!", syscall_id);
        }
    }
}
