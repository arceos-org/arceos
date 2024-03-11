//! 信号处理时保存的用户上下文。

/// 处理信号时使用的栈
///
/// 详细信息见`https://man7.org/linux/man-pages/man2/sigaltstack.2.html`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SignalStack {
    sp: usize,
    flags: u32,
    size: usize,
}

impl Default for SignalStack {
    fn default() -> Self {
        Self {
            sp: 0,
            // 代表SS_DISABLE，即不使用栈
            flags: 2,
            size: 0,
        }
    }
}
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// The `mcontext` struct for the signal action
pub struct MContext {
    // gregs
    r8: usize,
    r9: usize,
    r10: usize,
    r11: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
    rdi: usize,
    rsi: usize,
    rbp: usize,
    rbx: usize,
    rdx: usize,
    rax: usize,
    rcx: usize,
    rsp: usize,
    rip: usize,
    eflags: usize,
    cs: u16,
    gs: u16,
    fs: u16,
    _pad: u16,
    err: usize,
    trapno: usize,
    oldmask: usize,
    cr2: usize,
    // fpregs
    // TODO
    fpstate: usize,
    // reserved
    _reserved1: [usize; 8],
}

impl MContext {
    fn init_by_pc(pc: usize) -> Self {
        Self {
            rip: pc,
            ..Default::default()
        }
    }

    fn get_pc(&self) -> usize {
        self.rip
    }
}

#[repr(C)]
#[derive(Clone)]
/// The user context saved for the signal action, which can be accessed by the signal handler
pub struct SignalUserContext {
    flags: usize,
    link: usize,
    stack: SignalStack,
    mcontext: MContext,
    sigmask: u64,
    _fpregs: [usize; 64],
}

impl SignalUserContext {
    /// init the user context by the pc and the mask
    pub fn init(pc: usize, mask: usize) -> Self {
        Self {
            flags: 0,
            link: 0,
            stack: SignalStack::default(),
            mcontext: MContext::init_by_pc(pc),
            sigmask: mask as u64,
            _fpregs: [0; 64],
        }
    }

    /// get the pc from the user context
    pub fn get_pc(&self) -> usize {
        self.mcontext.get_pc()
    }
}
