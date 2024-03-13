//! To define the signal action and its flags

use crate::signal_no::SignalNo::{self, *};

/// 特殊取值，代表默认处理函数
pub const SIG_DFL: usize = 0;

/// 特殊取值，代表忽略这个信号
pub const SIG_IGN: usize = 1;

bitflags::bitflags! {
    #[allow(missing_docs)]
    #[derive(Default,Clone, Copy, Debug)]
    /// The flags of the signal action
    pub struct SigActionFlags: u32 {
        /// do not receive notification when child processes stop
        const SA_NOCLDSTOP = 1;
        /// do not create zombie on child process exit
        const SA_NOCLDWAIT = 2;
        /// use signal handler with 3 arguments, and sa_sigaction should be set instead of sa_handler.
        const SA_SIGINFO = 4;
        /// call the signal handler on an alternate signal stack provided by `sigaltstack(2)`
        const SA_ONSTACK = 0x08000000;
        /// restart system calls if possible
        const SA_RESTART = 0x10000000;
        /// do not automatically block the signal when its handler is being executed
        const SA_NODEFER = 0x40000000;
        /// restore the signal action to the default upon entry to the signal handler
        const SA_RESETHAND = 0x80000000;
        /// use the restorer field as the signal trampoline
        const SA_RESTORER = 0x4000000;
    }
}

/// 没有显式指定处理函数时的默认行为
pub enum SignalDefault {
    /// 终止进程
    Terminate,
    /// 忽略信号
    Ignore,
    /// 终止进程并转储核心，即程序当时的内存状态记录下来，保存在一个文件中，但当前未实现保存，直接退出进程
    Core,
    /// 暂停进程执行
    Stop,
    /// 恢复进程执行
    Cont,
}

impl SignalDefault {
    /// Get the default action of a signal
    pub fn get_action(signal: SignalNo) -> Self {
        match signal {
            SIGABRT => Self::Core,
            SIGALRM => Self::Terminate,
            SIGBUS => Self::Core,
            SIGCHLD => Self::Ignore,
            SIGCONT => Self::Cont,
            SIGFPE => Self::Core,
            SIGHUP => Self::Terminate,
            SIGILL => Self::Core,
            SIGINT => Self::Terminate,
            SIGKILL => Self::Terminate,
            SIGPIPE => Self::Terminate,
            SIGQUIT => Self::Core,
            SIGSEGV => Self::Core,
            SIGSTOP => Self::Stop,
            SIGTERM => Self::Terminate,
            SIGTSTP => Self::Stop,
            SIGTTIN => Self::Stop,
            SIGTTOU => Self::Stop,
            SIGUSR1 => Self::Terminate,
            SIGUSR2 => Self::Terminate,
            SIGXCPU => Self::Core,
            SIGXFSZ => Self::Core,
            SIGVTALRM => Self::Terminate,
            SIGPROF => Self::Terminate,
            SIGWINCH => Self::Ignore,
            SIGIO => Self::Terminate,
            SIGPWR => Self::Terminate,
            SIGSYS => Self::Core,
            _ => Self::Terminate,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// The structure of the signal action
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
    /// get the restorer address of the signal action
    ///
    /// When the SA_RESTORER flag is set, the restorer address is valid
    ///
    /// or it will return None, and the core will set the restore address as the signal trampoline
    pub fn get_storer(&self) -> Option<usize> {
        if self.sa_flags.contains(SigActionFlags::SA_RESTORER) {
            Some(self.restorer)
        } else {
            None
        }
    }
}
