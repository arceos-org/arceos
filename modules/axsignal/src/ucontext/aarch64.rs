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
#[derive(Clone, Debug)]
pub struct MContext {
    pub fault_address: usize,
    pub regs: [usize; 31],
    pub sp: usize,
    pub pc: usize,
    pub pstate: usize,
    pub reserved: [usize; 256*2],
}

impl Default for MContext {
    fn default() -> Self {
        Self {
            fault_address: 0,
            regs: [0; 31],
            sp: 0,
            pc: 0,
            pstate: 0,
            reserved: [0; 512],
        }
    }
}

impl MContext {
    fn init_by_pc(pc: usize) -> Self {
        Self {
            fault_address: 0,
            regs: [0; 31],
            sp: 0,
            pc: pc,
            pstate: 0,
            reserved: [0; 512],
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct SignalUserContext {
    pub flags: usize,
    pub link: usize,
    pub stack: SignalStack,
    pub sigmask: [usize; 17],
    pub mcontext: MContext,
}

impl SignalUserContext {
    pub fn init(pc: usize, _mask: usize) -> Self {
        Self {
            flags: 0,
            link: 0,
            stack: SignalStack::default(),
            mcontext: MContext::init_by_pc(pc),
            sigmask: [0; 17],
        }
    }

    pub fn get_pc(&self) -> usize {
        self.mcontext.pc
    }
}
