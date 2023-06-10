use core::arch::asm;
use memory_addr::VirtAddr;
use riscv::register::sstatus::{self, Sstatus};

include_asm_marcos!();

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
    /// 设置用户栈指针
    fn set_user_sp(&mut self, user_sp: usize) {
        self.regs.sp = user_sp;
    }
    /// 用于第一次进入应用程序时的初始化
    pub fn app_init_context(app_entry: usize, user_sp: usize) -> Self {
        let sstatus = sstatus::read();
        // 当前版本的riscv不支持使用set_spp函数，需要手动修改
        // 修改当前的sstatus为User，即是第8位置0
        let mut trap_frame = TrapFrame::default();
        trap_frame.set_user_sp(user_sp);
        trap_frame.sepc = app_entry;
        trap_frame.sstatus = unsafe { *(&sstatus as *const Sstatus as *const usize) & !(1 << 8) };
        unsafe {
            // a0为参数个数
            // a1存储的是用户栈底，即argv
            trap_frame.regs.a0 = *(user_sp as *const usize);
            trap_frame.regs.a1 = *(user_sp as *const usize).add(1) as usize;
        }
        trap_frame
    }
    /// 获取寄存器的值
    pub fn get_reg(&self, index: usize) -> usize {
        match index {
            1 => self.regs.ra,
            2 => self.regs.sp,
            3 => self.regs.gp,
            4 => self.regs.tp,
            5 => self.regs.t0,
            6 => self.regs.t1,
            7 => self.regs.t2,
            8 => self.regs.s0,
            9 => self.regs.s1,
            10 => self.regs.a0,
            11 => self.regs.a1,
            12 => self.regs.a2,
            13 => self.regs.a3,
            14 => self.regs.a4,
            15 => self.regs.a5,
            16 => self.regs.a6,
            17 => self.regs.a7,
            18 => self.regs.s2,
            19 => self.regs.s3,
            20 => self.regs.s4,
            21 => self.regs.s5,
            22 => self.regs.s6,
            23 => self.regs.s7,
            24 => self.regs.s8,
            25 => self.regs.s9,
            26 => self.regs.s10,
            27 => self.regs.s11,
            28 => self.regs.t3,
            29 => self.regs.t4,
            30 => self.regs.t5,
            31 => self.regs.t6,
            32 => self.sepc,
            33 => self.sstatus,
            _ => panic!("invalid register index"),
        }
    }
    /// 设置寄存器的值
    pub fn set_reg(&mut self, index: usize, value: usize) {
        match index {
            1 => self.regs.ra = value,
            2 => self.regs.sp = value,
            3 => self.regs.gp = value,
            4 => self.regs.tp = value,
            5 => self.regs.t0 = value,
            6 => self.regs.t1 = value,
            7 => self.regs.t2 = value,
            8 => self.regs.s0 = value,
            9 => self.regs.s1 = value,
            10 => self.regs.a0 = value,
            11 => self.regs.a1 = value,
            12 => self.regs.a2 = value,
            13 => self.regs.a3 = value,
            14 => self.regs.a4 = value,
            15 => self.regs.a5 = value,
            16 => self.regs.a6 = value,
            17 => self.regs.a7 = value,
            18 => self.regs.s2 = value,
            19 => self.regs.s3 = value,
            20 => self.regs.s4 = value,
            21 => self.regs.s5 = value,
            22 => self.regs.s6 = value,
            23 => self.regs.s7 = value,
            24 => self.regs.s8 = value,
            25 => self.regs.s9 = value,
            26 => self.regs.s10 = value,
            27 => self.regs.s11 = value,
            28 => self.regs.t3 = value,
            29 => self.regs.t4 = value,
            30 => self.regs.t5 = value,
            31 => self.regs.t6 = value,
            32 => self.sepc = value,
            33 => self.sstatus = value,
            _ => panic!("invalid register index"),
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
    // TODO: FP states
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn new_empty() -> *mut TaskContext {
        let task_ctx = TaskContext::new();
        let task_ctx_ptr = &task_ctx as *const TaskContext as *mut TaskContext;
        task_ctx_ptr
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe {
            // TODO: switch TLS
            context_switch(self, next_ctx)
        }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    asm!(
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
        options(noreturn),
    )
}
