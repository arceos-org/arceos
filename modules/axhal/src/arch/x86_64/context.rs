use core::{arch::naked_asm, fmt};
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
    /// Gets the 0th syscall argument.
    pub const fn arg0(&self) -> usize {
        self.rdi as _
    }

    /// Sets the 0th syscall argument.
    pub const fn set_arg0(&mut self, rdi: usize) {
        self.rdi = rdi as _;
    }

    /// Gets the 1st syscall argument.
    pub const fn arg1(&self) -> usize {
        self.rsi as _
    }

    /// Sets the 1st syscall argument.
    pub const fn set_arg1(&mut self, rsi: usize) {
        self.rsi = rsi as _;
    }

    /// Gets the 2nd syscall argument.
    pub const fn arg2(&self) -> usize {
        self.rdx as _
    }

    /// Sets the 2nd syscall argument.
    pub const fn set_arg2(&mut self, rdx: usize) {
        self.rdx = rdx as _;
    }

    /// Gets the 3rd syscall argument.
    pub const fn arg3(&self) -> usize {
        self.r10 as _
    }

    /// Sets the 3rd syscall argument.
    pub const fn set_arg3(&mut self, r10: usize) {
        self.r10 = r10 as _;
    }

    /// Gets the 4th syscall argument.
    pub const fn arg4(&self) -> usize {
        self.r8 as _
    }

    /// Sets the 4th syscall argument.
    pub const fn set_arg4(&mut self, r8: usize) {
        self.r8 = r8 as _;
    }

    /// Gets the 5th syscall argument.
    pub const fn arg5(&self) -> usize {
        self.r9 as _
    }

    /// Sets the 5th syscall argument.
    pub const fn set_arg5(&mut self, r9: usize) {
        self.r9 = r9 as _;
    }

    /// Whether the trap is from userspace.
    pub const fn is_user(&self) -> bool {
        self.cs & 0b11 == 3
    }

    /// Gets the instruction pointer.
    pub const fn ip(&self) -> usize {
        self.rip as _
    }

    /// Sets the instruction pointer.
    pub const fn set_ip(&mut self, rip: usize) {
        self.rip = rip as _;
    }

    /// Gets the stack pointer.
    pub const fn sp(&self) -> usize {
        self.rsp as _
    }

    /// Sets the stack pointer.
    pub const fn set_sp(&mut self, rsp: usize) {
        self.rsp = rsp as _;
    }

    /// Gets the return value register.
    pub const fn retval(&self) -> usize {
        self.rax as _
    }

    /// Sets the return value register.
    pub const fn set_retval(&mut self, rax: usize) {
        self.rax = rax as _;
    }

    /// Push the return address.
    ///
    /// On x86_64, return address is stored in stack, so we need to modify the
    /// stack in order to change the return address. This function uses a
    /// separate name (rather than `set_ra`) to avoid confusion and misuse.
    pub fn push_ra(&mut self, addr: usize) {
        self.rsp -= 8;
        unsafe {
            core::ptr::write(self.rsp as *mut usize, addr);
        }
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
        use crate::arch::GdtStruct;
        use x86_64::registers::rflags::RFlags;
        Self(TrapFrame {
            rdi: arg0 as _,
            rip: entry as _,
            cs: GdtStruct::UCODE64_SELECTOR.0 as _,
            #[cfg(feature = "irq")]
            rflags: RFlags::INTERRUPT_FLAG.bits(), // IOPL = 0, IF = 1
            rsp: ustack_top.as_usize() as _,
            ss: GdtStruct::UDATA_SELECTOR.0 as _,
            ..Default::default()
        })
    }

    /// Creates a new context from the given [`TrapFrame`].
    ///
    /// It copies almost all registers except `CS` and `SS` which need to be
    /// set to the user segment selectors.
    pub const fn from(tf: &TrapFrame) -> Self {
        use crate::arch::GdtStruct;
        let mut tf = *tf;
        tf.cs = GdtStruct::UCODE64_SELECTOR.0 as _;
        tf.ss = GdtStruct::UDATA_SELECTOR.0 as _;
        Self(tf)
    }

    /// Enters user space.
    ///
    /// It restores the user registers and jumps to the user entry point
    /// (saved in `rip`).
    /// When an exception or syscall occurs, the kernel stack pointer is
    /// switched to `kstack_top`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it changes processor mode and the stack.
    pub unsafe fn enter_uspace(&self, kstack_top: VirtAddr) -> ! {
        super::disable_irqs();
        assert_eq!(super::tss_get_rsp0(), kstack_top);
        unsafe {
            core::arch::asm!("
                mov     rsp, {tf}
                pop     rax
                pop     rcx
                pop     rdx
                pop     rbx
                pop     rbp
                pop     rsi
                pop     rdi
                pop     r8
                pop     r9
                pop     r10
                pop     r11
                pop     r12
                pop     r13
                pop     r14
                pop     r15
                add     rsp, 16     // skip vector, error_code
                swapgs
                iretq",
                tf = in(reg) &self.0,
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
    /// Thread Local Storage (TLS).
    pub fs_base: usize,
    /// The `gs_base` register value.
    #[cfg(feature = "uspace")]
    pub gs_base: usize,
    /// Extended states, i.e., FP/SIMD states.
    #[cfg(feature = "fp_simd")]
    pub ext_state: ExtendedState,
    /// The `CR3` register value, i.e., the page table root.
    #[cfg(feature = "uspace")]
    pub cr3: memory_addr::PhysAddr,
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
            kstack_top: va!(0),
            rsp: 0,
            fs_base: 0,
            #[cfg(feature = "uspace")]
            cr3: crate::paging::kernel_page_table_root(),
            #[cfg(feature = "fp_simd")]
            ext_state: ExtendedState::default(),
            #[cfg(feature = "uspace")]
            gs_base: 0,
        }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        unsafe {
            // x86_64 calling convention: the stack must be 16-byte aligned before
            // calling a function. That means when entering a new task (`ret` in `context_switch`
            // is executed), (stack pointer + 8) should be 16-byte aligned.
            let frame_ptr = (kstack_top.as_mut_ptr() as *mut u64).sub(1);
            let frame_ptr = (frame_ptr as *mut ContextSwitchFrame).sub(1);
            core::ptr::write(frame_ptr, ContextSwitchFrame {
                rip: entry as _,
                ..Default::default()
            });
            self.rsp = frame_ptr as u64;
        }
        self.kstack_top = kstack_top;
        self.fs_base = tls_area.as_usize();
    }

    /// Gets the TLS area.
    pub fn tls(&self) -> VirtAddr {
        VirtAddr::from(self.fs_base)
    }

    /// Sets the TLS area.
    pub fn set_tls(&mut self, tls_area: VirtAddr) {
        self.fs_base = tls_area.as_usize();
    }

    /// Changes the page table root (`CR3` register for x86_64).
    ///
    /// If not set, the kernel page table root is used (obtained by
    /// [`axhal::paging::kernel_page_table_root`][1]).
    ///
    /// [1]: crate::paging::kernel_page_table_root
    #[cfg(feature = "uspace")]
    pub fn set_page_table_root(&mut self, cr3: memory_addr::PhysAddr) {
        self.cr3 = cr3;
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
        #[cfg(any(feature = "tls", feature = "uspace"))]
        unsafe {
            self.fs_base = super::read_thread_pointer();
            super::write_thread_pointer(next_ctx.fs_base);
        }
        #[cfg(feature = "uspace")]
        unsafe {
            // Switch gs base for user space.
            self.gs_base = x86::msr::rdmsr(x86::msr::IA32_KERNEL_GSBASE) as usize;
            x86::msr::wrmsr(x86::msr::IA32_KERNEL_GSBASE, next_ctx.gs_base as u64);
            super::tss_set_rsp0(next_ctx.kstack_top);
            if next_ctx.cr3 != self.cr3 {
                super::write_page_table_root(next_ctx.cr3);
            }
        }
        unsafe { context_switch(&mut self.rsp, &next_ctx.rsp) }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_stack: &mut u64, _next_stack: &u64) {
    unsafe {
        naked_asm!(
            "
            .code64
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
        )
    }
}
