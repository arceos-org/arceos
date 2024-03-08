//! 信号处理时保存的用户上下文。

/// 处理信号时使用的栈
///
/// 详细信息见`https://man7.org/linux/man-pages/man2/sigaltstack.2.html`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SignalStack {
    pub sp: usize,
    pub flags: u32,
    pub size: usize,
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
pub struct MContext {
    // gregs
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rbp: usize,
    pub rbx: usize,
    pub rdx: usize,
    pub rax: usize,
    pub rcx: usize,
    pub rsp: usize,
    pub rip: usize,
    pub eflags: usize,
    pub cs: u16,
    pub gs: u16,
    pub fs: u16,
    pub _pad: u16,
    pub err: usize,
    pub trapno: usize,
    pub oldmask: usize,
    pub cr2: usize,
    // fpregs
    // TODO
    pub fpstate: usize,
    // reserved
    pub _reserved1: [usize; 8],
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
pub struct SignalUserContext {
    pub flags: usize,
    pub link: usize,
    pub stack: SignalStack,
    pub mcontext: MContext,
    pub sigmask: u64,
    pub _fpregs: [usize; 64],
}

impl SignalUserContext {
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

    pub fn get_pc(&self) -> usize {
        self.mcontext.get_pc()
    }
}
