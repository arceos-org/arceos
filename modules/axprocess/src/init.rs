/// 初始化进程的trap上下文
#[cfg(feature = "user")]
pub fn init_process() -> ! {
    extern "Rust" {
        fn __user_start();
    }
    use axhal::arch::TrapFrame;
    const STACK_SIZE: usize = 4096;
    static USERSTACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    static KERNELSTACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let trap_frame = TrapFrame::app_init_context(
        __user_start as usize,
        USERSTACK.as_ptr() as usize + STACK_SIZE,
    );
    // copy from trap.S
    let frame_address = &trap_frame as *const TrapFrame;
    let kernel_sp = KERNELSTACK.as_ptr() as usize + STACK_SIZE;
    unsafe {
        core::arch::asm!(
            r"
            mv      sp, {frame_base}
            LDR     gp, sp, 2                   // load user gp and tp
            LDR     t0, sp, 3
            STR     tp, sp, 3                   // save supervisor tp
            mv      tp, t0                      // tp：线程指针
            csrw    sscratch, {kernel_sp}       // put supervisor sp to scratch
            LDR     t0, sp, 31
            LDR     t1, sp, 32
            csrw    sepc, t0
            csrw    sstatus, t1
            POP_GENERAL_REGS
            LDR     sp, sp, 1
            sret
        ",
            frame_base = in(reg) frame_address,
            kernel_sp = in(reg) kernel_sp,
        );
    };
    core::panic!("already in user mode!")
}

#[cfg(not(feature = "user"))]
pub fn init_process() -> ! {
    extern "Rust" {
        fn main();
    }

    unsafe { main() };
    axtask::exit(0)
}
