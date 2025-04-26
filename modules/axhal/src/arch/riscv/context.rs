use core::arch::naked_asm;
use memory_addr::VirtAddr;
#[cfg(feature = "fp_simd")]
use riscv::register::sstatus::FS;

/// General registers of RISC-V.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GeneralRegisters {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize, // only valid for user traps
    pub tp: usize, // only valid for user traps
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

/// Floating-point registers of RISC-V.
#[cfg(feature = "fp_simd")]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FpStatus {
    /// the state of the RISC-V Floating-Point Unit (FPU)
    pub fp: [u64; 32],
    pub fcsr: usize,
    pub fs: FS,
}

#[cfg(feature = "fp_simd")]
impl Default for FpStatus {
    fn default() -> Self {
        Self {
            fs: FS::Initial,
            fp: [0; 32],
            fcsr: 0,
        }
    }
}

/// Saved registers when a trap (interrupt or exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    /// All general registers.
    pub regs: GeneralRegisters,
    /// Supervisor Exception Program Counter.
    pub sepc: usize,
    /// Supervisor Status Register.
    pub sstatus: usize,
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

    /// Sets the 1th syscall argument.
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
        self.sepc
    }

    /// Sets the instruction pointer.
    pub const fn set_ip(&mut self, pc: usize) {
        self.sepc = pc;
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
    pub const fn empty() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    /// Creates a new context with the given entry point, user stack pointer,
    /// and the argument.
    pub fn new(entry: usize, ustack_top: VirtAddr, arg0: usize) -> Self {
        const BIT_SPIE: usize = 5;
        const BIT_SUM: usize = 18;

        let mut sstatus: usize = 0;
        sstatus |= 1 << BIT_SPIE;
        sstatus |= 1 << BIT_SUM;
        #[cfg(feature = "fp_simd")]
        {
            // set the initial state of the FPU
            const BIT_FS: usize = 13;
            sstatus |= (FS::Initial as usize) << BIT_FS;
        }

        Self(TrapFrame {
            regs: GeneralRegisters {
                a0: arg0,
                sp: ustack_top.as_usize(),
                ..Default::default()
            },
            sepc: entry,
            sstatus,
        })
    }

    /// Creates a new context from the given [`TrapFrame`].
    pub const fn from(trap_frame: &TrapFrame) -> Self {
        Self(*trap_frame)
    }

    /// Enters user space.
    ///
    /// It restores the user registers and jumps to the user entry point
    /// (saved in `sepc`).
    /// When an exception or syscall occurs, the kernel stack pointer is
    /// switched to `kstack_top`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it changes processor mode and the stack.
    #[unsafe(no_mangle)]
    pub unsafe fn enter_uspace(&self, kstack_top: VirtAddr) -> ! {
        use riscv::register::{sepc, sscratch};

        super::disable_irqs();
        sscratch::write(kstack_top.as_usize());
        sepc::write(self.0.sepc);
        // Address of the top of the kernel stack after saving the trap frame.
        let kernel_trap_addr = kstack_top.as_usize() - core::mem::size_of::<TrapFrame>();
        unsafe {
            core::arch::asm!(
                include_asm_macros!(),
                "
                mv      sp, {tf}

                STR     gp, {kernel_trap_addr}, 2
                LDR     gp, sp, 2

                STR     tp, {kernel_trap_addr}, 3
                LDR     tp, sp, 3

                LDR     t0, sp, 32
                csrw    sstatus, t0
                POP_GENERAL_REGS
                LDR     sp, sp, 1
                sret",
                tf = in(reg) &(self.0),
                kernel_trap_addr = in(reg) kernel_trap_addr,
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
/// - Thread pointer register (for kernel-space thread-local storage)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    /// Thread pointer
    pub tp: usize,
    /// The `satp` register value, i.e., the page table root.
    #[cfg(feature = "uspace")]
    pub satp: memory_addr::PhysAddr,
    #[cfg(feature = "fp_simd")]
    pub fp_status: FpStatus,
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
        Self {
            #[cfg(feature = "uspace")]
            satp: crate::paging::kernel_page_table_root(),
            #[cfg(feature = "fp_simd")]
            fp_status: FpStatus {
                fs: FS::Initial,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.tp = tls_area.as_usize();
    }

    /// Changes the page table root (`satp` register for riscv64).
    ///
    /// If not set, the kernel page table root is used (obtained by
    /// [`axhal::paging::kernel_page_table_root`][1]).
    ///
    /// [1]: crate::paging::kernel_page_table_root
    #[cfg(feature = "uspace")]
    pub fn set_page_table_root(&mut self, satp: memory_addr::PhysAddr) {
        self.satp = satp;
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
        unsafe {
            if self.satp != next_ctx.satp {
                super::write_page_table_root(next_ctx.satp);
            }
        }
        #[cfg(feature = "fp_simd")]
        {
            use riscv::register::sstatus;
            use riscv::register::sstatus::FS;
            // get the real FP state of the current task
            let current_fs = sstatus::read().fs();
            // save the current task's FP state
            if current_fs == FS::Dirty {
                // we need to save the current task's FP state
                unsafe {
                    save_fp_registers(&mut self.fp_status.fp);
                }
                // after saving, we set the FP state to clean
                self.fp_status.fs = FS::Clean;
            }
            // restore the next task's FP state
            match next_ctx.fp_status.fs {
                FS::Clean => unsafe {
                    // the next task's FP state is clean, we should restore it
                    restore_fp_registers(&next_ctx.fp_status.fp);
                    // after restoring, we set the FP state
                    sstatus::set_fs(FS::Clean);
                },
                FS::Initial => unsafe {
                    // restore the FP state as constant values(all 0)
                    clear_fp_registers();
                    // we set the FP state to initial
                    sstatus::set_fs(FS::Initial);
                },
                FS::Dirty => {
                    // should not happen, since we set FS to Clean after saving
                    panic!("FP state of the next task should not be dirty");
                }
                _ => {}
            }
        }

        unsafe { context_switch(self, next_ctx) }
    }
}

#[cfg(feature = "fp_simd")]
#[naked]
unsafe extern "C" fn save_fp_registers(_fp_registers: &mut [u64; 32]) {
    naked_asm!(
        include_fp_asm_macros!(),
        "
        PUSH_FLOAT_REGS a0
        frcsr t0
        STR t0, a0, 32
        ret
        "
    )
}

#[cfg(feature = "fp_simd")]
#[naked]
unsafe extern "C" fn restore_fp_registers(_fp_registers: &[u64; 32]) {
    naked_asm!(
        include_fp_asm_macros!(),
        "
        POP_FLOAT_REGS a0
        LDR t0, a0, 32
        fscsr x0, t0
        ret
        "
    )
}

#[cfg(feature = "fp_simd")]
#[naked]
unsafe extern "C" fn clear_fp_registers() {
    naked_asm!(
        include_fp_asm_macros!(),
        "
        CLEAR_FLOAT_REGS
        ret
        "
    )
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    naked_asm!(
        include_asm_macros!(),
        "
        // save old context (callee-saved registers)
        STR     ra, a0, 0
        STR     sp, a0, 1
        STR     s0, a0, 2
        STR     s1, a0, 3
        STR     s2, a0, 4
        STR     s3, a0, 5
        STR     s4, a0, 6
        STR     s5, a0, 7
        STR     s6, a0, 8
        STR     s7, a0, 9
        STR     s8, a0, 10
        STR     s9, a0, 11
        STR     s10, a0, 12
        STR     s11, a0, 13

        // restore new context
        LDR     s11, a1, 13
        LDR     s10, a1, 12
        LDR     s9, a1, 11
        LDR     s8, a1, 10
        LDR     s7, a1, 9
        LDR     s6, a1, 8
        LDR     s5, a1, 7
        LDR     s4, a1, 6
        LDR     s3, a1, 5
        LDR     s2, a1, 4
        LDR     s1, a1, 3
        LDR     s0, a1, 2
        LDR     sp, a1, 1
        LDR     ra, a1, 0

        ret",
    )
}
