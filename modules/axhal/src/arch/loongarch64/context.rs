use core::arch::asm;
use memory_addr::VirtAddr;

/// Saved registers when a trap (interrupt or exception) occurs.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct TrapFrame {
    /// All general registers.
    pub regs: [usize; 32],
    /// Pre-exception Mode Information
    pub prmd: usize,
    /// Exception Return Address
    pub era: usize,
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
        unsafe { context_switch(self, next_ctx) }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    asm!(
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
        options(noreturn),
    )
}
