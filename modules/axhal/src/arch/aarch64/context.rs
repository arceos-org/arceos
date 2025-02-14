use core::arch::naked_asm;
use memory_addr::VirtAddr;

/// Saved registers when a trap (exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    /// General-purpose registers (R0..R30).
    pub r: [u64; 31],
    /// User Stack Pointer (SP_EL0).
    pub usp: u64,
    /// Exception Link Register (ELR_EL1).
    pub elr: u64,
    /// Saved Process Status Register (SPSR_EL1).
    pub spsr: u64,
}

impl TrapFrame {
    /// Gets the 0th syscall argument.
    pub const fn arg0(&self) -> usize {
        self.r[0] as _
    }

    /// Gets the 1st syscall argument.
    pub const fn arg1(&self) -> usize {
        self.r[1] as _
    }

    /// Gets the 2nd syscall argument.
    pub const fn arg2(&self) -> usize {
        self.r[2] as _
    }

    /// Gets the 3rd syscall argument.
    pub const fn arg3(&self) -> usize {
        self.r[3] as _
    }

    /// Gets the 4th syscall argument.
    pub const fn arg4(&self) -> usize {
        self.r[4] as _
    }

    /// Gets the 5th syscall argument.
    pub const fn arg5(&self) -> usize {
        self.r[5] as _
    }
}

/// Context to enter user space.
#[cfg(feature = "uspace")]
pub struct UspaceContext(TrapFrame);

#[cfg(feature = "uspace")]
impl UspaceContext {
    /// Creates an empty context with all registers set to zero.
    pub const fn empty() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Creates a new context with the given entry point, user stack pointer,
    /// and the argument.
    pub fn new(entry: usize, ustack_top: VirtAddr, arg0: usize) -> Self {
        use aarch64_cpu::registers::SPSR_EL1;
        let mut regs = [0; 31];
        regs[0] = arg0 as _;
        Self(TrapFrame {
            r: regs,
            usp: ustack_top.as_usize() as _,
            elr: entry as _,
            spsr: (SPSR_EL1::M::EL0t
                + SPSR_EL1::D::Masked
                + SPSR_EL1::A::Masked
                + SPSR_EL1::I::Unmasked
                + SPSR_EL1::F::Masked)
                .value,
        })
    }

    /// Creates a new context from the given [`TrapFrame`].
    pub const fn from(trap_frame: &TrapFrame) -> Self {
        Self(*trap_frame)
    }

    /// Gets the instruction pointer.
    pub const fn get_ip(&self) -> usize {
        self.0.elr as _
    }

    /// Gets the stack pointer.
    pub const fn get_sp(&self) -> usize {
        self.0.usp as _
    }

    /// Sets the instruction pointer.
    pub const fn set_ip(&mut self, pc: usize) {
        self.0.elr = pc as _;
    }

    /// Sets the stack pointer.
    pub const fn set_sp(&mut self, sp: usize) {
        self.0.usp = sp as _;
    }

    /// Sets the return value register.
    pub const fn set_retval(&mut self, r0: usize) {
        self.0.r[0] = r0 as _;
    }

    /// Enters user space.
    ///
    /// It restores the user registers and jumps to the user entry point
    /// (saved in `elr`).
    /// When an exception or syscall occurs, the kernel stack pointer is
    /// switched to `kstack_top`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it changes processor mode and the stack.
    #[inline(never)]
    #[unsafe(no_mangle)]
    pub unsafe fn enter_uspace(&self, kstack_top: VirtAddr) -> ! {
        super::disable_irqs();
        // We do not handle traps that occur at the current exception level,
        // so the kstack ptr(`sp_el1`) will not change during running in user space.
        // Then we don't need to save the `sp_el1` to the taskctx.
        unsafe {
            core::arch::asm!(
                "
                mov     sp, x1
                ldp     x30, x9, [x0, 30 * 8]
                ldp     x10, x11, [x0, 32 * 8]
                msr     sp_el0, x9
                msr     elr_el1, x10
                msr     spsr_el1, x11

                ldp     x28, x29, [x0, 28 * 8]
                ldp     x26, x27, [x0, 26 * 8]
                ldp     x24, x25, [x0, 24 * 8]
                ldp     x22, x23, [x0, 22 * 8]
                ldp     x20, x21, [x0, 20 * 8]
                ldp     x18, x19, [x0, 18 * 8]
                ldp     x16, x17, [x0, 16 * 8]
                ldp     x14, x15, [x0, 14 * 8]
                ldp     x12, x13, [x0, 12 * 8]
                ldp     x10, x11, [x0, 10 * 8]
                ldp     x8, x9, [x0, 8 * 8]
                ldp     x6, x7, [x0, 6 * 8]
                ldp     x4, x5, [x0, 4 * 8]
                ldp     x2, x3, [x0, 2 * 8]
                ldp     x0, x1, [x0]
                eret",
                in("x0") &self.0,
                in("x1") kstack_top.as_usize() ,
                options(noreturn),
            )
        }
    }
}

/// FP & SIMD registers.
#[repr(C, align(16))]
#[derive(Debug, Default)]
pub struct FpState {
    /// 128-bit SIMD & FP registers (V0..V31)
    pub regs: [u128; 32],
    /// Floating-point Control Register (FPCR)
    pub fpcr: u32,
    /// Floating-point Status Register (FPSR)
    pub fpsr: u32,
}

#[cfg(feature = "fp_simd")]
impl FpState {
    fn switch_to(&mut self, next_fpstate: &FpState) {
        unsafe { fpstate_switch(self, next_fpstate) }
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
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub sp: u64,
    pub tpidr_el0: u64,
    pub r19: u64,
    pub r20: u64,
    pub r21: u64,
    pub r22: u64,
    pub r23: u64,
    pub r24: u64,
    pub r25: u64,
    pub r26: u64,
    pub r27: u64,
    pub r28: u64,
    pub r29: u64,
    pub lr: u64, // r30
    /// The `ttbr0_el1` register value, i.e., the page table root.
    #[cfg(feature = "uspace")]
    pub ttbr0_el1: memory_addr::PhysAddr,
    #[cfg(feature = "fp_simd")]
    pub fp_state: FpState,
}

impl TaskContext {
    /// Creates a dummy context for a new task.
    ///
    /// Note the context is not initialized, it will be filled by [`switch_to`]
    /// (for initial tasks) and [`init`] (for regular tasks) methods.
    ///
    /// [`init`]: TaskContext::init
    /// [`switch_to`]: TaskContext::switch_to
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.sp = kstack_top.as_usize() as u64;
        self.lr = entry as u64;
        // When under `uspace` feature, kernel will not use this register.
        self.tpidr_el0 = tls_area.as_usize() as u64;
    }

    /// Changes the page table root for user space (`ttbr0_el1` register for aarch64 in el1 level).
    ///
    /// If not set, it means that this task is a kernel task and only `ttbr1_el1` register will be used.
    #[cfg(feature = "uspace")]
    pub fn set_page_table_root(&mut self, ttbr0_el1: memory_addr::PhysAddr) {
        self.ttbr0_el1 = ttbr0_el1;
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        #[cfg(feature = "fp_simd")]
        self.fp_state.switch_to(&next_ctx.fp_state);
        #[cfg(feature = "uspace")]
        {
            if self.ttbr0_el1 != next_ctx.ttbr0_el1 {
                unsafe { super::write_page_table_root0(next_ctx.ttbr0_el1) };
            }
        }
        unsafe { context_switch(self, next_ctx) }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    naked_asm!(
        "
        // save old context (callee-saved registers)
        stp     x29, x30, [x0, 12 * 8]
        stp     x27, x28, [x0, 10 * 8]
        stp     x25, x26, [x0, 8 * 8]
        stp     x23, x24, [x0, 6 * 8]
        stp     x21, x22, [x0, 4 * 8]
        stp     x19, x20, [x0, 2 * 8]
        mov     x19, sp
        mrs     x20, tpidr_el0
        stp     x19, x20, [x0]

        // restore new context
        ldp     x19, x20, [x1]
        mov     sp, x19
        msr     tpidr_el0, x20
        ldp     x19, x20, [x1, 2 * 8]
        ldp     x21, x22, [x1, 4 * 8]
        ldp     x23, x24, [x1, 6 * 8]
        ldp     x25, x26, [x1, 8 * 8]
        ldp     x27, x28, [x1, 10 * 8]
        ldp     x29, x30, [x1, 12 * 8]

        ret",
    )
}

#[naked]
#[cfg(feature = "fp_simd")]
unsafe extern "C" fn fpstate_switch(_current_fpstate: &mut FpState, _next_fpstate: &FpState) {
    naked_asm!(
        "
        // save fp/neon context
        mrs     x9, fpcr
        mrs     x10, fpsr
        stp     q0, q1, [x0, 0 * 16]
        stp     q2, q3, [x0, 2 * 16]
        stp     q4, q5, [x0, 4 * 16]
        stp     q6, q7, [x0, 6 * 16]
        stp     q8, q9, [x0, 8 * 16]
        stp     q10, q11, [x0, 10 * 16]
        stp     q12, q13, [x0, 12 * 16]
        stp     q14, q15, [x0, 14 * 16]
        stp     q16, q17, [x0, 16 * 16]
        stp     q18, q19, [x0, 18 * 16]
        stp     q20, q21, [x0, 20 * 16]
        stp     q22, q23, [x0, 22 * 16]
        stp     q24, q25, [x0, 24 * 16]
        stp     q26, q27, [x0, 26 * 16]
        stp     q28, q29, [x0, 28 * 16]
        stp     q30, q31, [x0, 30 * 16]
        str     x9, [x0, 64 *  8]
        str     x10, [x0, 65 * 8]

        // restore fp/neon context
        ldp     q0, q1, [x1, 0 * 16]
        ldp     q2, q3, [x1, 2 * 16]
        ldp     q4, q5, [x1, 4 * 16]
        ldp     q6, q7, [x1, 6 * 16]
        ldp     q8, q9, [x1, 8 * 16]
        ldp     q10, q11, [x1, 10 * 16]
        ldp     q12, q13, [x1, 12 * 16]
        ldp     q14, q15, [x1, 14 * 16]
        ldp     q16, q17, [x1, 16 * 16]
        ldp     q18, q19, [x1, 18 * 16]
        ldp     q20, q21, [x1, 20 * 16]
        ldp     q22, q23, [x1, 22 * 16]
        ldp     q24, q25, [x1, 24 * 16]
        ldp     q26, q27, [x1, 26 * 16]
        ldp     q28, q29, [x1, 28 * 16]
        ldp     q30, q31, [x1, 30 * 16]
        ldr     x9, [x1, 64 * 8]
        ldr     x10, [x1, 65 * 8]
        msr     fpcr, x9
        msr     fpsr, x10

        isb
        ret",
    )
}
