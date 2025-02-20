use core::arch::naked_asm;
use memory_addr::VirtAddr;
/// Saved registers when a trap (interrupt or exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    /// All general registers.
    pub regs: [usize; 32],
    /// Pre-exception Mode Information
    pub prmd: usize,
    /// Exception Return Address
    pub era: usize,
    /// Access Memory Address When Exception
    pub badv: usize,
    /// Current Mode Information
    pub crmd: usize,
}

impl TrapFrame {
    /// Gets the 0th syscall argument.
    pub const fn arg0(&self) -> usize {
        self.regs[4] as _
    }

    /// Gets the 1st syscall argument.
    pub const fn arg1(&self) -> usize {
        self.regs[5] as _
    }

    /// Gets the 2nd syscall argument.
    pub const fn arg2(&self) -> usize {
        self.regs[6] as _
    }

    /// Gets the 3rd syscall argument.
    pub const fn arg3(&self) -> usize {
        self.regs[7] as _
    }

    /// Gets the 4th syscall argument.
    pub const fn arg4(&self) -> usize {
        self.regs[8] as _
    }

    /// Gets the 5th syscall argument.
    pub const fn arg5(&self) -> usize {
        self.regs[9] as _
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
        let mut trap_frame = TrapFrame::default();
        const PPLV_UMODE: usize = 0b11;
        const PIE: usize = 1 << 2;
        trap_frame.regs[3] = ustack_top.as_usize();
        trap_frame.era = entry;
        trap_frame.prmd = PPLV_UMODE | PIE;
        trap_frame.regs[4] = arg0;
        Self(trap_frame)
    }

    /// Creates a new context from the given [`TrapFrame`].
    pub const fn from(trap_frame: &TrapFrame) -> Self {
        Self(*trap_frame)
    }

    /// Gets the instruction pointer.
    pub const fn get_ip(&self) -> usize {
        self.0.era
    }

    /// Gets the stack pointer.
    pub const fn get_sp(&self) -> usize {
        self.0.regs[3]
    }

    /// Sets the instruction pointer.
    pub const fn set_ip(&mut self, pc: usize) {
        self.0.era = pc;
    }

    /// Sets the stack pointer.
    pub const fn set_sp(&mut self, sp: usize) {
        self.0.regs[3] = sp;
    }

    /// Sets the return value register.
    pub const fn set_retval(&mut self, a0: usize) {
        self.0.regs[4] = a0;
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
        use loongArch64::register::{CpuMode, era, prmd};

        super::disable_irqs();
        prmd::set_pplv(CpuMode::Ring3);
        prmd::set_pie(true);
        era::set_pc(self.get_ip());

        unsafe {
            core::arch::asm!(
                r"
                .equ KSAVE_KSP,    0x30
                .equ KSAVE_T0,     0x31
                .equ KSAVE_USP,    0x32
                .equ KSAVE_R21,    0x33
                .equ KSAVE_TP,     0x34
                .equ LA_CSR_EUEN,  0x2

                move      $sp, {tf}
                csrwr     $tp,  KSAVE_TP
                csrwr     $r21, KSAVE_R21
    
                csrwr     {kstack_top}, KSAVE_KSP           // save ksp into SAVE0 CSR
                
                ld.d      $r1, $sp, 1*8
                ld.d      $tp, $sp, 2*8
                ld.d      $r4, $sp, 4*8
                ld.d      $r5, $sp, 5*8
                ld.d      $r6, $sp, 6*8
                ld.d      $r7, $sp, 7*8
                ld.d      $r8, $sp, 8*8
                ld.d      $r9, $sp, 9*8
                ld.d      $r10, $sp, 10*8
                ld.d      $r11, $sp, 11*8
                ld.d      $r12, $sp, 12*8
                ld.d      $r13, $sp, 13*8
                ld.d      $r14, $sp, 14*8
                ld.d      $r15, $sp, 15*8
                ld.d      $r16, $sp, 16*8
                ld.d      $r17, $sp, 17*8
                ld.d      $r18, $sp, 18*8
                ld.d      $r19, $sp, 19*8
                ld.d      $r20, $sp, 20*8
                ld.d      $r21, $sp, 21*8
                ld.d      $r22, $sp, 22*8
                ld.d      $r23, $sp, 23*8
                ld.d      $r24, $sp, 24*8
                ld.d      $r25, $sp, 25*8
                ld.d      $r26, $sp, 26*8
                ld.d      $r27, $sp, 27*8
                ld.d      $r28, $sp, 28*8
                ld.d      $r29, $sp, 29*8
                ld.d      $r30, $sp, 30*8
                ld.d      $r31, $sp, 31*8
    
                ld.d      $sp, $sp, 3*8       // user sp
                ertn
                ",
                tf = in (reg) &self.0,
                kstack_top = in(reg) kstack_top.as_usize(),
                options(noreturn),
            );
        }
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
    pub ra: usize,      // return address
    pub sp: usize,      // stack pointer
    pub s: [usize; 10], // loongArch need to save 10 static registers from $r22 to $r31
    pub tp: usize,
    #[cfg(feature = "uspace")]
    pub pgdl: usize,
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
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
    /// If not set, the kernel page table root is used (obtained by
    /// [`axhal::paging::kernel_page_table_root`][1]).
    ///
    /// [1]: crate::paging::kernel_page_table_root
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
        unsafe { context_switch(self, next_ctx) }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    unsafe {
        naked_asm!(
            "
            // save old context (callee-saved registers)
            st.d     $ra, $a0, 0
            st.d     $sp, $a0, 1 * 8
            st.d     $s0, $a0, 2 * 8
            st.d     $s1, $a0, 3 * 8
            st.d     $s2, $a0, 4 * 8
            st.d     $s3, $a0, 5 * 8
            st.d     $s4, $a0, 6 * 8
            st.d     $s5, $a0, 7 * 8
            st.d     $s6, $a0, 8 * 8
            st.d     $s7, $a0, 9 * 8
            st.d     $s8, $a0, 10 * 8
            st.d     $fp, $a0, 11 * 8
    
            // restore new context
            ld.d     $ra, $a1, 0
            ld.d     $s0, $a1, 2 * 8
            ld.d     $s1, $a1, 3 * 8
            ld.d     $s2, $a1, 4 * 8
            ld.d     $s3, $a1, 5 * 8
            ld.d     $s4, $a1, 6 * 8
            ld.d     $s5, $a1, 7 * 8
            ld.d     $s6, $a1, 8 * 8
            ld.d     $s7, $a1, 9 * 8
            ld.d     $s8, $a1, 10 * 8
            ld.d     $fp, $a1, 11 * 8
            ld.d     $sp, $a1, 1 * 8
    
            ret",
        )
    }
}
