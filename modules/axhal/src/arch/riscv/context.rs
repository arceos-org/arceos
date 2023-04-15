use core::arch::asm;
use memory_addr::VirtAddr;

include_asm_marcos!();

#[repr(C)]
#[derive(Debug, Default, Clone)]
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

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct TrapFrame {
    pub regs: GeneralRegisters,
    pub sepc: usize,
    pub sstatus: usize,
    #[cfg(feature = "user-paging")]
    pub kstack: usize,
    #[cfg(feature = "user-paging")]
    pub satp: usize,
    #[cfg(feature = "user-paging")]
    pub trap_handler: usize,
}

impl TrapFrame {

    #[cfg(feature = "user")]
    /// Create a trap frame with entry (sepc) and user stack
    /// specified
    pub fn new(entry: usize, ustack: usize) -> TrapFrame {
        use riscv::register::sstatus::{self, Sstatus};

        
        let mut trap_frame = TrapFrame::default();
        trap_frame.regs.sp = ustack;
        trap_frame.sepc = entry;
        
        let sstatus_reg = sstatus::read();
        // set up SPP (8th bit) to 0 (User)
        trap_frame.sstatus = unsafe {
            *(&sstatus_reg as *const Sstatus as *const usize) & !(1 << 8)
        };
        #[cfg(feature = "user-paging")]
        {
            use riscv::register::satp::{self, Satp};
            let satp_reg = satp::read();
            trap_frame.satp = unsafe {
                *(&satp_reg as *const Satp as *const usize)
            };
            extern "Rust" {
                fn riscv_trap_handler();
            }
            trap_frame.trap_handler = riscv_trap_handler as usize
        }
        
        trap_frame    
    }

    #[cfg(feature = "user")]
    /// Enter user space, with kstack specified
    pub fn enter_uspace(&self, sp: usize) -> ! { // sp: kernel trap space 
        unsafe {
            core::arch::asm!(r"
                mv      sp, {tf}
                //LDR     gp, sp, 2                   // load user gp and tp
                //LDR     t0, sp, 3
                //STR     tp, sp, 3                   // save supervisor tp
                //mv      tp, t0
                csrw    sscratch, {sp}

                LDR     t0, sp, 31
                LDR     t1, sp, 32
                csrw    sepc, t0
                csrw    sstatus, t1

                POP_GENERAL_REGS
                LDR     sp, sp, 1                   // load sp from tf.regs.sp
                sret
            ",
                sp = in(reg) sp,
                tf = in(reg) (self as *const TrapFrame),
            );
        };
        unreachable!("already in user space")
    }
}

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
}

impl TaskContext {
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
    }

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
