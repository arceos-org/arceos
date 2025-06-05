use core::arch::naked_asm;
#[cfg(feature = "fp_simd")]
use core::mem::offset_of;
use memory_addr::VirtAddr;

/// General registers of Loongarch64.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GeneralRegisters {
    pub zero: usize,
    pub ra: usize,
    pub tp: usize,
    pub sp: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub t8: usize,
    pub u0: usize,
    pub fp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
}

/// Floating-point registers of LoongArch64.
#[cfg(feature = "fp_simd")]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct FpStatus {
    /// Floating-point registers (f0-f31)
    pub fp: [u64; 32],
    /// Floating-point Condition Code register
    pub fcc: [u8; 8],
    /// Floating-point Control and Status register
    pub fcsr: usize,
}

/// Saved registers when a trap (interrupt or exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    /// All general registers.
    pub regs: GeneralRegisters,
    /// Pre-exception Mode Information
    pub prmd: usize,
    /// Exception Return Address
    pub era: usize,
}

impl TrapFrame {
    /// Gets the 0th syscall argument.
    pub const fn arg0(&self) -> usize {
        self.regs.a0
    }

    /// Sets the 0th syscall argument.
    pub const fn set_arg0(&mut self, a0: usize) {
        self.regs.a0 = a0;
    }

    /// Gets the 1st syscall argument.
    pub const fn arg1(&self) -> usize {
        self.regs.a1
    }

    /// Sets the 1st syscall argument.
    pub const fn set_arg1(&mut self, a1: usize) {
        self.regs.a1 = a1;
    }

    /// Gets the 2nd syscall argument.
    pub const fn arg2(&self) -> usize {
        self.regs.a2
    }

    /// Sets the 2nd syscall argument.
    pub const fn set_arg2(&mut self, a2: usize) {
        self.regs.a2 = a2;
    }

    /// Gets the 3rd syscall argument.
    pub const fn arg3(&self) -> usize {
        self.regs.a3
    }

    /// Sets the 3rd syscall argument.
    pub const fn set_arg3(&mut self, a3: usize) {
        self.regs.a3 = a3;
    }

    /// Gets the 4th syscall argument.
    pub const fn arg4(&self) -> usize {
        self.regs.a4
    }

    /// Sets the 4th syscall argument.
    pub const fn set_arg4(&mut self, a4: usize) {
        self.regs.a4 = a4;
    }

    /// Gets the 5th syscall argument.
    pub const fn arg5(&self) -> usize {
        self.regs.a5
    }

    /// Sets the 5th syscall argument.
    pub const fn set_arg5(&mut self, a5: usize) {
        self.regs.a5 = a5;
    }

    /// Gets the instruction pointer.
    pub const fn ip(&self) -> usize {
        self.era
    }

    /// Sets the instruction pointer.
    pub const fn set_ip(&mut self, pc: usize) {
        self.era = pc;
    }

    /// Gets the stack pointer.
    pub const fn sp(&self) -> usize {
        self.regs.sp
    }

    /// Sets the stack pointer.
    pub const fn set_sp(&mut self, sp: usize) {
        self.regs.sp = sp;
    }

    /// Gets the return value register.
    pub const fn retval(&self) -> usize {
        self.regs.a0
    }

    /// Sets the return value register.
    pub const fn set_retval(&mut self, a0: usize) {
        self.regs.a0 = a0;
    }

    /// Sets the return address.
    pub const fn set_ra(&mut self, ra: usize) {
        self.regs.ra = ra;
    }

    /// Gets the TLS area.
    pub const fn tls(&self) -> usize {
        self.regs.tp
    }

    /// Sets the TLS area.
    pub const fn set_tls(&mut self, tls_area: usize) {
        self.regs.tp = tls_area;
    }
}

/// Context to enter user space.
#[cfg(feature = "uspace")]
pub struct UspaceContext(TrapFrame);

#[cfg(feature = "uspace")]
impl UspaceContext {
    /// Creates an empty context with all registers set to zero.
    pub fn empty() -> Self {
        Self(Default::default())
    }

    /// Creates a new context with the given entry point, user stack pointer,
    /// and the argument.
    pub fn new(entry: usize, ustack_top: VirtAddr, arg0: usize) -> Self {
        let mut trap_frame = TrapFrame::default();
        const PPLV_UMODE: usize = 0b11;
        const PIE: usize = 1 << 2;
        trap_frame.regs.sp = ustack_top.as_usize();
        trap_frame.era = entry;
        trap_frame.prmd = PPLV_UMODE | PIE;
        trap_frame.regs.a0 = arg0;
        Self(trap_frame)
    }

    /// Creates a new context from the given [`TrapFrame`].
    pub const fn from(trap_frame: &TrapFrame) -> Self {
        Self(*trap_frame)
    }

    /// Enters user space.
    ///
    /// It restores the user registers and jumps to the user entry point
    /// (saved in `era`).
    /// When an exception or syscall occurs, the kernel stack pointer is
    /// switched to `kstack_top`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it changes processor mode and the stack.
    #[unsafe(no_mangle)]
    pub unsafe fn enter_uspace(&self, kstack_top: VirtAddr) -> ! {
        use loongArch64::register::era;

        super::disable_irqs();
        era::set_pc(self.ip());

        unsafe {
            core::arch::asm!(
                include_asm_macros!(),
                "
                move      $sp, {tf}
                csrwr     $tp,  KSAVE_TP
                csrwr     $r21, KSAVE_R21
                LDD       $tp,  $sp, 32
                csrwr     $tp,  LA_CSR_PRMD
                csrwr     {kstack_top}, KSAVE_KSP           // save ksp into SAVE0 CSR

                POP_GENERAL_REGS

                LDD      $tp,   $sp, 2
                LDD      $r21,  $sp, 21
                LDD      $sp,   $sp, 3       // user sp
                ertn",
                tf = in (reg) &self.0,
                kstack_top = in(reg) kstack_top.as_usize(),
                options(noreturn),
            )
        }
    }
}

#[cfg(feature = "uspace")]
impl core::ops::Deref for UspaceContext {
    type Target = TrapFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "uspace")]
impl core::ops::DerefMut for UspaceContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for kernel space thread-local storage)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    /// Return Address
    pub ra: usize,
    /// Stack Pointer
    pub sp: usize,
    /// loongArch need to save 10 static registers from $r22 to $r31
    pub s: [usize; 10],
    /// Thread Pointer
    pub tp: usize,
    #[cfg(feature = "uspace")]
    /// user page table root
    pub pgdl: usize,
    #[cfg(feature = "fp_simd")]
    /// Floating Point Status
    pub fp_status: FpStatus,
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub fn new() -> Self {
        Default::default()
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.tp = tls_area.as_usize();
    }

    /// Changes the page table root (`pgdl` register for loongarch64).
    ///
    /// If not set, it means that this task is a kernel task and only `pgdh` register will be used.
    #[cfg(feature = "uspace")]
    pub fn set_page_table_root(&mut self, pgdl: memory_addr::PhysAddr) {
        self.pgdl = pgdl.as_usize();
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        #[cfg(feature = "tls")]
        {
            self.tp = super::read_thread_pointer();
            unsafe { super::write_thread_pointer(next_ctx.tp) };
        }
        #[cfg(feature = "uspace")]
        {
            if self.pgdl != next_ctx.pgdl {
                unsafe { super::write_page_table_root0(pa!(next_ctx.pgdl)) };
            }
        }
        #[cfg(feature = "fp_simd")]
        unsafe {
            save_fp_registers(&mut self.fp_status);
            restore_fp_registers(&next_ctx.fp_status);
        }

        unsafe { context_switch(self, next_ctx) }
    }
}

#[cfg(feature = "fp_simd")]
#[unsafe(naked)]
unsafe extern "C" fn save_fp_registers(fp_status: &mut FpStatus) {
    naked_asm!(
        include_fp_asm_macros!(),
        "
        PUSH_FLOAT_REGS $a0
        addi.d $t8, $a0, {fcc_offset}
        SAVE_FCC $t8
        addi.d $t8, $a0, {fcsr_offset}
        SAVE_FCSR $t8
        ret",
        fcc_offset = const offset_of!(FpStatus, fcc),
        fcsr_offset = const offset_of!(FpStatus, fcsr),
    )
}

#[cfg(feature = "fp_simd")]
#[unsafe(naked)]
unsafe extern "C" fn restore_fp_registers(fp_status: &FpStatus) {
    naked_asm!(
        include_fp_asm_macros!(),
        "
        POP_FLOAT_REGS $a0
        addi.d $t8, $a0, {fcc_offset}
        RESTORE_FCC $t8
        addi.d $t8, $a0, {fcsr_offset}
        RESTORE_FCSR $t8
        ret",
        fcc_offset = const offset_of!(FpStatus, fcc),
        fcsr_offset = const offset_of!(FpStatus, fcsr),
    )
}

#[unsafe(naked)]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    naked_asm!(
        include_asm_macros!(),
        "
        // save old context (callee-saved registers)
        STD     $ra, $a0, 0
        STD     $sp, $a0, 1
        STD     $s0, $a0, 2
        STD     $s1, $a0, 3
        STD     $s2, $a0, 4
        STD     $s3, $a0, 5
        STD     $s4, $a0, 6
        STD     $s5, $a0, 7
        STD     $s6, $a0, 8
        STD     $s7, $a0, 9
        STD     $s8, $a0, 10
        STD     $fp, $a0, 11

        // restore new context
        LDD     $fp, $a1, 11
        LDD     $s8, $a1, 10
        LDD     $s7, $a1, 9
        LDD     $s6, $a1, 8
        LDD     $s5, $a1, 7
        LDD     $s4, $a1, 6
        LDD     $s3, $a1, 5
        LDD     $s2, $a1, 4
        LDD     $s1, $a1, 3
        LDD     $s0, $a1, 2
        LDD     $sp, $a1, 1
        LDD     $ra, $a1, 0

        ret",
    )
}
