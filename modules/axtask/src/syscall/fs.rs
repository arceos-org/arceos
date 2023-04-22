/// 处理关于输出输入的系统调用
use axhal::console::write_bytes;
#[allow(unused)]
const STDIN: usize = 0;
const STDOUT: usize = 1;

pub fn syscall_write(fd: usize, buf: *const u8, len: usize) -> isize {
    //暂时不考虑文件调用，因此直接输出到标准输出
    match fd {
        STDOUT => {
            let answer = unsafe { core::slice::from_raw_parts(buf, len) };
            write_bytes(answer);
            0
        }
        _ => {
            panic!("Invalid fd:{}", fd);
        }
    }
}
