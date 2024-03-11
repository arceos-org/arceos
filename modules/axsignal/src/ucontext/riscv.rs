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
#[derive(Clone, Debug)]
/// The `mcontext` struct for the signal action
pub struct MContext {
    reserved1: [usize; 16],
    pc: usize,
    reserved2: [usize; 17],
    fpstate: [usize; 66],
}

impl Default for MContext {
    fn default() -> Self {
        Self {
            reserved1: [0; 16],
            pc: 0,
            reserved2: [0; 17],
            fpstate: [0; 66],
        }
    }
}

impl MContext {
    fn init_by_pc(pc: usize) -> Self {
        Self {
            reserved1: [0; 16],
            pc,
            reserved2: [0; 17],
            fpstate: [0; 66],
        }
    }

    fn get_pc(&self) -> usize {
        self.pc
    }
}

#[repr(C)]
#[derive(Clone)]
/// The user context saved for the signal action, which can be accessed by the signal handler
pub struct SignalUserContext {
    flags: usize,
    link: usize,
    stack: SignalStack,
    sigmask: u64,
    mcontext: MContext,
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
        }
    }

    /// get the pc from the user context
    pub fn get_pc(&self) -> usize {
        self.mcontext.get_pc()
    }
}
