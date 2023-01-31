use core::{arch::asm, fmt};
use memory_addr::VirtAddr;

#[repr(C)]
#[derive(Debug, Default, Clone)]
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

    // Pushed by 'vector.S'
    pub vector: u64,
    pub error_code: u64,

    // Pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,

    // Pushed by CPU when trap from ring-3
    pub user_rsp: u64,
    pub user_ss: u64,
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

#[repr(align(16))]
pub struct ExtentedState {
    fxsave_area: [u8; 512],
}

impl ExtentedState {
    #[inline]
    fn save(&mut self) {
        unsafe { core::arch::x86_64::_fxsave64(self.fxsave_area.as_mut_ptr()) }
    }

    #[inline]
    fn restore(&self) {
        unsafe { core::arch::x86_64::_fxrstor64(self.fxsave_area.as_ptr()) }
    }
}

impl fmt::Debug for ExtentedState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ExtentedState")
            .field("fxsave_area", unsafe {
                &core::mem::transmute::<_, [u128; 32]>(self.fxsave_area)
            })
            .finish()
    }
}

#[derive(Debug)]
pub struct TaskContext {
    pub kstack_top: VirtAddr,
    pub rsp: u64,
    pub ext_state: ExtentedState,
}

impl TaskContext {
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

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

    pub fn switch_to(&mut self, next_ctx: &Self) {
        if cfg!(target_feature = "sse") {
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
