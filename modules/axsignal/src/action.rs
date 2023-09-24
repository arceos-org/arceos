pub const SIGNAL_RETURN_TRAP: usize = 0xFFFF_FF80_0000_0000;

use crate::signal_no::SignalNo;

/// 特殊取值，代表默认处理函数
pub const SIG_DFL: usize = 0;

/// 特殊取值，代表忽略这个信号
pub const SIG_IGN: usize = 1;

bitflags::bitflags! {
    #[derive(Default,Clone, Copy)]
    pub struct SigActionFlags: usize {
        const SA_NOCLDSTOP = 1;
        const SA_NOCLDWAIT = 2;
        const SA_SIGINFO = 4;
        const SA_ONSTACK = 0x08000000;
        const SA_RESTART = 0x10000000;
        const SA_NODEFER = 0x40000000;
        const SA_RESETHAND = 0x80000000;
        const SA_RESTORER = 0x04000000;
    }
}

/// 没有显式指定处理函数时的默认行为
pub enum SignalDefault {
    /// 终止进程
    Terminate,
    /// 忽略信号
    Ignore,
}

impl SignalDefault {
    pub fn get_action(signal: SignalNo) -> Self {
        match signal {
            SignalNo::SIGCHLD | SignalNo::SIGURG => SignalDefault::Ignore,
            _ => SignalDefault::Terminate,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SigAction {
    /// 信号处理函数的地址
    /// 1. 如果是上述特殊值 SIG_DFL 或 SIG_IGN，则按描述处理
    /// 2. 若flags没有指定SA_SIGINFO，则函数原型为 fn(sig: SignalNo) -> ()，对应C语言原型为 void (*sa_handler)(int)
    /// 3. 若flags指定了SA_SIGINFO，则函数原型为 fn(sig: SignalNo, info: &SigInfo, ucontext: &mut UContext) -> ()，
    /// 对应C语言原型为 void (*sa_sigaction)(int, siginfo_t *, void *)。
    ///
    /// 其中，SigInfo和SignalNo的定义见siginfo.rs和signal_no.rs。
    /// UContext即是处理信号时内核保存的用户态上下文，它存储在用户地址空间，会在调用sig_return时被恢复，定义见ucontext.rs。
    pub sa_handler: usize,
    /// 信号处理的flags
    pub sa_flags: SigActionFlags,
    /// 信号处理的跳板页地址，存储了sig_return的函数处理地址
    /// 仅在SA_RESTORER标志被设置时有效
    pub restorer: usize,
    /// 该信号处理函数的信号掩码
    pub sa_mask: usize,
}

impl SigAction {
    pub fn get_storer(&self) -> usize {
        if self.sa_flags.contains(SigActionFlags::SA_RESTORER) {
            self.restorer
        } else {
            SIGNAL_RETURN_TRAP
        }
    }
}
