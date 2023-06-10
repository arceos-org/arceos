use core::{arch::asm, fmt};
use memory_addr::VirtAddr;

/// Saved registers when a trap (interrupt or exception) occurs.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    // Pushed by `trap.S`
    pub vector: u64,
    pub error_code: u64,

    // Pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl TrapFrame {
    /// Whether the trap is from userspace.
    pub const fn is_user(&self) -> bool {
        self.cs & 0b11 == 3
    }

    /// Initialize the context for a new process.
    pub fn app_init_context(_app_entry: usize, _user_sp: usize) -> Self {
        Self::default()
    }

    /// 获取寄存器的值
    pub fn get_reg(&self, index: usize) -> usize {
        match index {
            1 => self.rax as usize,
            2 => self.rdi as usize,
            3 => self.rsi as usize,
            4 => self.rdx as usize,
            5 => self.r10 as usize,
            6 => self.r8 as usize,
            7 => self.r9 as usize,
            8 => self.r15 as usize,
            9 => self.r14 as usize,
            10 => self.r13 as usize,
            11 => self.r12 as usize,
            12 => self.r11 as usize,
            13 => self.rbp as usize,
            14 => self.rbx as usize,
            15 => self.rcx as usize,
            16 => self.rsp as usize,
            17 => self.rip as usize,
            18 => self.rflags as usize,
            19 => self.cs as usize,
            20 => self.ss as usize,
            _ => panic!("get_reg: invalid index {}", index),
        }
    }

    /// 设置寄存器的值
    pub fn set_reg(&mut self, index: usize, value: usize) {
        match index {
            1 => self.rax = value as u64,
            2 => self.rdi = value as u64,
            3 => self.rsi = value as u64,
            4 => self.rdx = value as u64,
            5 => self.r10 = value as u64,
            6 => self.r8 = value as u64,
            7 => self.r9 = value as u64,
            8 => self.r15 = value as u64,
            9 => self.r14 = value as u64,
            10 => self.r13 = value as u64,
            11 => self.r12 = value as u64,
            12 => self.r11 = value as u64,
            13 => self.rbp = value as u64,
            14 => self.rbx = value as u64,
            15 => self.rcx = value as u64,
            16 => self.rsp = value as u64,
            17 => self.rip = value as u64,
            18 => self.rflags = value as u64,
            19 => self.cs = value as u64,
            20 => self.ss = value as u64,
            _ => panic!("set_reg: invalid index {}", index),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct ContextSwitchFrame {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
    rip: u64,
}

/// A 512-byte memory region for the FXSAVE/FXRSTOR instruction to save and
/// restore the x87 FPU, MMX, XMM, and MXCSR registers.
///
/// See <https://www.felixcloutier.com/x86/fxsave> for more details.
#[allow(missing_docs)]
#[repr(C, align(16))]
#[derive(Debug)]
pub struct FxsaveArea {
    pub fcw: u16,
    pub fsw: u16,
    pub ftw: u16,
    pub fop: u16,
    pub fip: u64,
    pub fdp: u64,
    pub mxcsr: u32,
    pub mxcsr_mask: u32,
    pub st: [u64; 16],
    pub xmm: [u64; 32],
    _padding: [u64; 12],
}

static_assertions::const_assert_eq!(core::mem::size_of::<FxsaveArea>(), 512);

/// Extended state of a task, such as FP/SIMD states.
pub struct ExtendedState {
    /// Memory region for the FXSAVE/FXRSTOR instruction.
    pub fxsave_area: FxsaveArea,
}

#[cfg(feature = "fp_simd")]
impl ExtendedState {
    #[inline]
    fn save(&mut self) {
        unsafe { core::arch::x86_64::_fxsave64(&mut self.fxsave_area as *mut _ as *mut u8) }
    }

    #[inline]
    fn restore(&self) {
        unsafe { core::arch::x86_64::_fxrstor64(&self.fxsave_area as *const _ as *const u8) }
    }

    const fn default() -> Self {
        let mut area: FxsaveArea = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        area.fcw = 0x37f;
        area.ftw = 0xffff;
        area.mxcsr = 0x1f80;
        Self { fxsave_area: area }
    }
}

impl fmt::Debug for ExtendedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtendedState")
            .field("fxsave_area", &self.fxsave_area)
            .finish()
    }
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
///
/// On x86_64, callee-saved registers are saved to the kernel stack by the
/// `PUSH` instruction. So that [`rsp`] is the `RSP` after callee-saved
/// registers are pushed, and [`kstack_top`] is the top of the kernel stack
/// (`RSP` before any push).
///
/// [`rsp`]: TaskContext::rsp
/// [`kstack_top`]: TaskContext::kstack_top
#[derive(Debug)]
pub struct TaskContext {
    /// The kernel stack top of the task.
    pub kstack_top: VirtAddr,
    /// `RSP` after all callee-saved registers are pushed.
    pub rsp: u64,
    /// Extended states, i.e., FP/SIMD states.
    #[cfg(feature = "fp_simd")]
    pub ext_state: ExtendedState,
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        Self {
            kstack_top: VirtAddr::from(0),
            rsp: 0,
            #[cfg(feature = "fp_simd")]
            ext_state: ExtendedState::default(),
        }
    }

    /// Creates a new empty context for a new task.
    pub fn new_empty() -> *mut TaskContext {
        let task_ctx = TaskContext::new();
        let task_ctx_ptr = &task_ctx as *const TaskContext as *mut TaskContext;
        task_ctx_ptr
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr) {
        unsafe {
            // x86_64 calling convention: the stack must be 16-byte aligned before
            // calling a function. That means when entering a new task (`ret` in `context_switch`
            // is executed), (stack pointer + 8) should be 16-byte aligned.
            let frame_ptr = (kstack_top.as_mut_ptr() as *mut u64).sub(1);
            let frame_ptr = (frame_ptr as *mut ContextSwitchFrame).sub(1);
            core::ptr::write(
                frame_ptr,
                ContextSwitchFrame {
                    rip: entry as _,
                    ..Default::default()
                },
            );
            self.rsp = frame_ptr as u64;
        }
        self.kstack_top = kstack_top;
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        #[cfg(feature = "fp_simd")]
        {
            self.ext_state.save();
            next_ctx.ext_state.restore();
        }
        unsafe {
            // TODO: swtich tls
            context_switch(&mut self.rsp, &next_ctx.rsp)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_stack: &mut u64, _next_stack: &u64) {
    asm!(
        "
        push    rbp
        push    rbx
        push    r12
        push    r13
        push    r14
        push    r15
        mov     [rdi], rsp

        mov     rsp, [rsi]
        pop     r15
        pop     r14
        pop     r13
        pop     r12
        pop     rbx
        pop     rbp
        ret",
        options(noreturn),
    )
}
