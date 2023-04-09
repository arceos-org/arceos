#![no_std]
#![no_main]

extern crate axruntime;

pub fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!("ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x13") args[3],
            in("x14") args[4],
            in("x15") args[5],
            in("x17") id
        );
    }
    ret
}

#[no_mangle]
fn __user_start() {
    static print_str: &str = "hello world!";
    syscall(axruntime::sys_number::SYS_WRITE, [print_str.as_ptr() as usize, print_str.len(), 0, 0, 0, 0]);
    syscall(axruntime::sys_number::SYS_EXIT, [0, 0, 0, 0, 0, 0]);
}
