//! System Control Block

use core::ptr;

#[cfg(not(armv6m))]
use cortex_m_0_7::peripheral::cpuid::CsselrCacheType;
#[cfg(not(armv6m))]
use super::CBP;
#[cfg(not(armv6m))]
use super::CPUID;
use super::SCB;

pub use cortex_m_0_7::peripheral::scb::{RegisterBlock};

#[cfg(has_fpu)]
pub use cortex_m_0_7::peripheral::scb::FpuAccessMode;

#[cfg(has_fpu)]
mod fpu_consts {
    pub const SCB_CPACR_FPU_MASK: u32 = 0b11_11 << 20;
    pub const SCB_CPACR_FPU_ENABLE: u32 = 0b01_01 << 20;
    pub const SCB_CPACR_FPU_USER: u32 = 0b10_10 << 20;
}

#[cfg(has_fpu)]
use self::fpu_consts::*;

#[cfg(has_fpu)]
impl SCB {
    /// Shorthand for `set_fpu_access_mode(FpuAccessMode::Disabled)`
    #[inline]
    pub fn disable_fpu(&mut self) {
        self.set_fpu_access_mode(FpuAccessMode::Disabled)
    }

    /// Shorthand for `set_fpu_access_mode(FpuAccessMode::Enabled)`
    #[inline]
    pub fn enable_fpu(&mut self) {
        self.set_fpu_access_mode(FpuAccessMode::Enabled)
    }

    /// Gets FPU access mode
    #[inline]
    pub fn fpu_access_mode() -> FpuAccessMode {
        // NOTE(unsafe) atomic read operation with no side effects
        let cpacr = unsafe { (*Self::ptr()).cpacr.read() };

        if cpacr & SCB_CPACR_FPU_MASK == SCB_CPACR_FPU_ENABLE | SCB_CPACR_FPU_USER {
            FpuAccessMode::Enabled
        } else if cpacr & SCB_CPACR_FPU_MASK == SCB_CPACR_FPU_ENABLE {
            FpuAccessMode::Privileged
        } else {
            FpuAccessMode::Disabled
        }
    }

    /// Sets FPU access mode
    ///
    /// *IMPORTANT* Any function that runs fully or partly with the FPU disabled must *not* take any
    /// floating-point arguments or have any floating-point local variables. Because the compiler
    /// might inline such a function into a caller that does have floating-point arguments or
    /// variables, any such function must be also marked #[inline(never)].
    #[inline]
    pub fn set_fpu_access_mode(&mut self, mode: FpuAccessMode) {
        let mut cpacr = self.cpacr.read() & !SCB_CPACR_FPU_MASK;
        match mode {
            FpuAccessMode::Disabled => (),
            FpuAccessMode::Privileged => cpacr |= SCB_CPACR_FPU_ENABLE,
            FpuAccessMode::Enabled => cpacr |= SCB_CPACR_FPU_ENABLE | SCB_CPACR_FPU_USER,
        }
        unsafe { self.cpacr.write(cpacr) }
    }
}

impl SCB {
    /// Returns the active exception number
    #[inline]
    pub fn vect_active() -> VectActive {
        let icsr = unsafe { ptr::read(&(*SCB::ptr()).icsr as *const _ as *const u32) };

        match icsr as u8 {
            0 => VectActive::ThreadMode,
            2 => VectActive::Exception(Exception::NonMaskableInt),
            3 => VectActive::Exception(Exception::HardFault),
            #[cfg(not(armv6m))]
            4 => VectActive::Exception(Exception::MemoryManagement),
            #[cfg(not(armv6m))]
            5 => VectActive::Exception(Exception::BusFault),
            #[cfg(not(armv6m))]
            6 => VectActive::Exception(Exception::UsageFault),
            #[cfg(any(armv8m, target_arch = "x86_64"))]
            7 => VectActive::Exception(Exception::SecureFault),
            11 => VectActive::Exception(Exception::SVCall),
            #[cfg(not(armv6m))]
            12 => VectActive::Exception(Exception::DebugMonitor),
            14 => VectActive::Exception(Exception::PendSV),
            15 => VectActive::Exception(Exception::SysTick),
            irqn => VectActive::Interrupt { irqn: irqn - 16 },
        }
    }
}

pub use cortex_m_0_7::peripheral::scb::{Exception, VectActive};

#[cfg(not(armv6m))]
mod scb_consts {
    pub const SCB_CCR_IC_MASK: u32 = 1 << 17;
    pub const SCB_CCR_DC_MASK: u32 = 1 << 16;
}

#[cfg(not(armv6m))]
use self::scb_consts::*;

#[cfg(not(armv6m))]
impl SCB {
    /// Enables I-Cache if currently disabled
    #[inline]
    pub fn enable_icache(&mut self) {
        // Don't do anything if ICache is already enabled
        if Self::icache_enabled() {
            return;
        }

        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        // Invalidate I-Cache
        cbp.iciallu();

        // Enable I-cache
        extern "C" {
            // see asm-v7m.s
            fn __enable_icache();
        }

        // NOTE(unsafe): The asm routine manages exclusive access to the SCB
        // registers and applies the proper barriers; it is technically safe on
        // its own, and is only `unsafe` here because it's `extern "C"`.
        unsafe { __enable_icache(); }
    }

    /// Disables I-Cache if currently enabled
    #[inline]
    pub fn disable_icache(&mut self) {
        // Don't do anything if ICache is already disabled
        if !Self::icache_enabled() {
            return;
        }

        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        // Disable I-Cache
        unsafe { self.ccr.modify(|r| r & !SCB_CCR_IC_MASK) };

        // Invalidate I-Cache
        cbp.iciallu();

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Returns whether the I-Cache is currently enabled
    #[inline]
    pub fn icache_enabled() -> bool {
        crate::asm::dsb();
        crate::asm::isb();

        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Self::ptr()).ccr.read() & SCB_CCR_IC_MASK == SCB_CCR_IC_MASK }
    }

    /// Invalidates I-Cache
    #[inline]
    pub fn invalidate_icache(&mut self) {
        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        // Invalidate I-Cache
        cbp.iciallu();

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Enables D-cache if currently disabled
    #[inline]
    pub fn enable_dcache(&mut self, cpuid: &mut CPUID) {
        // Don't do anything if DCache is already enabled
        if Self::dcache_enabled() {
            return;
        }

        // Invalidate anything currently in the DCache
        self.invalidate_dcache(cpuid);

        // Now turn on the D-cache
        extern "C" {
            // see asm-v7m.s
            fn __enable_dcache();
        }

        // NOTE(unsafe): The asm routine manages exclusive access to the SCB
        // registers and applies the proper barriers; it is technically safe on
        // its own, and is only `unsafe` here because it's `extern "C"`.
        unsafe { __enable_dcache(); }
    }

    /// Disables D-cache if currently enabled
    #[inline]
    pub fn disable_dcache(&mut self, cpuid: &mut CPUID) {
        // Don't do anything if DCache is already disabled
        if !Self::dcache_enabled() {
            return;
        }

        // Turn off the DCache
        unsafe { self.ccr.modify(|r| r & !SCB_CCR_DC_MASK) };

        // Clean and invalidate whatever was left in it
        self.clean_invalidate_dcache(cpuid);
    }

    /// Returns whether the D-Cache is currently enabled
    #[inline]
    pub fn dcache_enabled() -> bool {
        crate::asm::dsb();
        crate::asm::isb();

        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Self::ptr()).ccr.read() & SCB_CCR_DC_MASK == SCB_CCR_DC_MASK }
    }

    /// Invalidates D-cache
    ///
    /// Note that calling this while the dcache is enabled will probably wipe out your
    /// stack, depending on optimisations, breaking returning to the call point.
    /// It's used immediately before enabling the dcache, but not exported publicly.
    #[inline]
    fn invalidate_dcache(&mut self, cpuid: &mut CPUID) {
        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        // Read number of sets and ways
        let (sets, ways) = cpuid.cache_num_sets_ways(0, CsselrCacheType::DataOrUnified);

        // Invalidate entire D-Cache
        for set in 0..sets {
            for way in 0..ways {
                cbp.dcisw(set, way);
            }
        }

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Cleans D-cache
    #[inline]
    pub fn clean_dcache(&mut self, cpuid: &mut CPUID) {
        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        // Read number of sets and ways
        let (sets, ways) = cpuid.cache_num_sets_ways(0, CsselrCacheType::DataOrUnified);

        for set in 0..sets {
            for way in 0..ways {
                cbp.dccsw(set, way);
            }
        }

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Cleans and invalidates D-cache
    #[inline]
    pub fn clean_invalidate_dcache(&mut self, cpuid: &mut CPUID) {
        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        // Read number of sets and ways
        let (sets, ways) = cpuid.cache_num_sets_ways(0, CsselrCacheType::DataOrUnified);

        for set in 0..sets {
            for way in 0..ways {
                cbp.dccisw(set, way);
            }
        }

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Invalidates D-cache by address
    ///
    /// `addr`: the address to invalidate
    /// `size`: size of the memory block, in number of bytes
    ///
    /// Invalidates cache starting from the lowest 32-byte aligned address represented by `addr`,
    /// in blocks of 32 bytes until at least `size` bytes have been invalidated.
    #[inline]
    pub fn invalidate_dcache_by_address(&mut self, addr: usize, size: usize) {
        // No-op zero sized operations
        if size == 0 {
            return;
        }

        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        crate::asm::dsb();

        // Cache lines are fixed to 32 bit on Cortex-M7 and not present in earlier Cortex-M
        const LINESIZE: usize = 32;
        let num_lines = ((size - 1) / LINESIZE) + 1;

        let mut addr = addr & 0xFFFF_FFE0;

        for _ in 0..num_lines {
            cbp.dcimvac(addr as u32);
            addr += LINESIZE;
        }

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Cleans D-cache by address
    ///
    /// `addr`: the address to clean
    /// `size`: size of the memory block, in number of bytes
    ///
    /// Cleans cache starting from the lowest 32-byte aligned address represented by `addr`,
    /// in blocks of 32 bytes until at least `size` bytes have been cleaned.
    #[inline]
    pub fn clean_dcache_by_address(&mut self, addr: usize, size: usize) {
        // No-op zero sized operations
        if size == 0 {
            return;
        }

        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        crate::asm::dsb();

        // Cache lines are fixed to 32 bit on Cortex-M7 and not present in earlier Cortex-M
        const LINESIZE: usize = 32;
        let num_lines = ((size - 1) / LINESIZE) + 1;

        let mut addr = addr & 0xFFFF_FFE0;

        for _ in 0..num_lines {
            cbp.dccmvac(addr as u32);
            addr += LINESIZE;
        }

        crate::asm::dsb();
        crate::asm::isb();
    }

    /// Cleans and invalidates D-cache by address
    ///
    /// `addr`: the address to clean and invalidate
    /// `size`: size of the memory block, in number of bytes
    ///
    /// Cleans and invalidates cache starting from the lowest 32-byte aligned address represented
    /// by `addr`, in blocks of 32 bytes until at least `size` bytes have been cleaned and
    /// invalidated.
    #[inline]
    pub fn clean_invalidate_dcache_by_address(&mut self, addr: usize, size: usize) {
        // No-op zero sized operations
        if size == 0 {
            return;
        }

        // NOTE(unsafe) All CBP registers are write-only and stateless
        let mut cbp = unsafe { CBP::new() };

        crate::asm::dsb();

        // Cache lines are fixed to 32 bit on Cortex-M7 and not present in earlier Cortex-M
        const LINESIZE: usize = 32;
        let num_lines = ((size - 1) / LINESIZE) + 1;

        let mut addr = addr & 0xFFFF_FFE0;

        for _ in 0..num_lines {
            cbp.dccimvac(addr as u32);
            addr += LINESIZE;
        }

        crate::asm::dsb();
        crate::asm::isb();
    }
}

const SCB_SCR_SLEEPDEEP: u32 = 0x1 << 2;

impl SCB {
    /// Set the SLEEPDEEP bit in the SCR register
    #[inline]
    pub fn set_sleepdeep(&mut self) {
        unsafe {
            self.scr.modify(|scr| scr | SCB_SCR_SLEEPDEEP);
        }
    }

    /// Clear the SLEEPDEEP bit in the SCR register
    #[inline]
    pub fn clear_sleepdeep(&mut self) {
        unsafe {
            self.scr.modify(|scr| scr & !SCB_SCR_SLEEPDEEP);
        }
    }
}

const SCB_SCR_SLEEPONEXIT: u32 = 0x1 << 1;

impl SCB {
    /// Set the SLEEPONEXIT bit in the SCR register
    #[inline]
    pub fn set_sleeponexit(&mut self) {
        unsafe {
            self.scr.modify(|scr| scr | SCB_SCR_SLEEPONEXIT);
        }
    }

    /// Clear the SLEEPONEXIT bit in the SCR register
    #[inline]
    pub fn clear_sleeponexit(&mut self) {
        unsafe {
            self.scr.modify(|scr| scr & !SCB_SCR_SLEEPONEXIT);
        }
    }
}

const SCB_AIRCR_VECTKEY: u32 = 0x05FA << 16;
const SCB_AIRCR_PRIGROUP_MASK: u32 = 0x5 << 8;
const SCB_AIRCR_SYSRESETREQ: u32 = 1 << 2;

impl SCB {
    /// Initiate a system reset request to reset the MCU
    #[deprecated(since = "0.6.1", note = "Use `SCB::sys_reset`")]
    #[inline]
    pub fn system_reset(&mut self) -> ! {
        crate::asm::dsb();
        unsafe {
            self.aircr.modify(
                |r| {
                    SCB_AIRCR_VECTKEY | // otherwise the write is ignored
            r & SCB_AIRCR_PRIGROUP_MASK | // keep priority group unchanged
            SCB_AIRCR_SYSRESETREQ
                }, // set the bit
            )
        };
        crate::asm::dsb();
        loop {
            // wait for the reset
            crate::asm::nop(); // avoid rust-lang/rust#28728
        }
    }

    /// Initiate a system reset request to reset the MCU
    #[inline]
    pub fn sys_reset() -> ! {
        crate::asm::dsb();
        unsafe {
            (*Self::ptr()).aircr.modify(
                |r| {
                    SCB_AIRCR_VECTKEY | // otherwise the write is ignored
            r & SCB_AIRCR_PRIGROUP_MASK | // keep priority group unchanged
            SCB_AIRCR_SYSRESETREQ
                }, // set the bit
            )
        };
        crate::asm::dsb();
        loop {
            // wait for the reset
            crate::asm::nop(); // avoid rust-lang/rust#28728
        }
    }
}

const SCB_ICSR_PENDSVSET: u32 = 1 << 28;
const SCB_ICSR_PENDSVCLR: u32 = 1 << 27;

const SCB_ICSR_PENDSTSET: u32 = 1 << 26;
const SCB_ICSR_PENDSTCLR: u32 = 1 << 25;

impl SCB {
    /// Set the PENDSVSET bit in the ICSR register which will pend the PendSV interrupt
    #[inline]
    pub fn set_pendsv() {
        unsafe {
            (*Self::ptr()).icsr.write(SCB_ICSR_PENDSVSET);
        }
    }

    /// Check if PENDSVSET bit in the ICSR register is set meaning PendSV interrupt is pending
    #[inline]
    pub fn is_pendsv_pending() -> bool {
        unsafe { (*Self::ptr()).icsr.read() & SCB_ICSR_PENDSVSET == SCB_ICSR_PENDSVSET }
    }

    /// Set the PENDSVCLR bit in the ICSR register which will clear a pending PendSV interrupt
    #[inline]
    pub fn clear_pendsv() {
        unsafe {
            (*Self::ptr()).icsr.write(SCB_ICSR_PENDSVCLR);
        }
    }

    /// Set the PENDSTSET bit in the ICSR register which will pend a SysTick interrupt
    #[inline]
    pub fn set_pendst() {
        unsafe {
            (*Self::ptr()).icsr.write(SCB_ICSR_PENDSTSET);
        }
    }

    /// Check if PENDSTSET bit in the ICSR register is set meaning SysTick interrupt is pending
    #[inline]
    pub fn is_pendst_pending() -> bool {
        unsafe { (*Self::ptr()).icsr.read() & SCB_ICSR_PENDSTSET == SCB_ICSR_PENDSTSET }
    }

    /// Set the PENDSTCLR bit in the ICSR register which will clear a pending SysTick interrupt
    #[inline]
    pub fn clear_pendst() {
        unsafe {
            (*Self::ptr()).icsr.write(SCB_ICSR_PENDSTCLR);
        }
    }
}

/// System handlers, exceptions with configurable priority
#[allow(clippy::missing_inline_in_public_items)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemHandler {
    // NonMaskableInt, // priority is fixed
    // HardFault, // priority is fixed
    /// Memory management interrupt (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    MemoryManagement,

    /// Bus fault interrupt (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    BusFault,

    /// Usage fault interrupt (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    UsageFault,

    /// Secure fault interrupt (only on ARMv8-M)
    #[cfg(any(armv8m, target_arch = "x86_64"))]
    SecureFault,

    /// SV call interrupt
    SVCall,

    /// Debug monitor interrupt (not present on Cortex-M0 variants)
    #[cfg(not(armv6m))]
    DebugMonitor,

    /// Pend SV interrupt
    PendSV,

    /// System Tick interrupt
    SysTick,
}

impl SystemHandler {
    fn index(self) -> u8 {
        match self {
            #[cfg(not(armv6m))]
            SystemHandler::MemoryManagement => 4,
            #[cfg(not(armv6m))]
            SystemHandler::BusFault => 5,
            #[cfg(not(armv6m))]
            SystemHandler::UsageFault => 6,
            #[cfg(any(armv8m, target_arch = "x86_64"))]
            SystemHandler::SecureFault => 7,
            SystemHandler::SVCall => 11,
            #[cfg(not(armv6m))]
            SystemHandler::DebugMonitor => 12,
            SystemHandler::PendSV => 14,
            SystemHandler::SysTick => 15,
        }
    }
}

impl SCB {
    /// Returns the hardware priority of `system_handler`
    ///
    /// *NOTE*: Hardware priority does not exactly match logical priority levels. See
    /// [`NVIC.get_priority`](struct.NVIC.html#method.get_priority) for more details.
    #[inline]
    pub fn get_priority(system_handler: SystemHandler) -> u8 {
        let index = system_handler.index();

        #[cfg(not(armv6m))]
        {
            // NOTE(unsafe) atomic read with no side effects
            unsafe { (*Self::ptr()).shpr[usize::from(index - 4)].read() }
        }

        #[cfg(armv6m)]
        {
            // NOTE(unsafe) atomic read with no side effects
            let shpr = unsafe { (*Self::ptr()).shpr[usize::from((index - 8) / 4)].read() };
            let prio = (shpr >> (8 * (index % 4))) & 0x0000_00ff;
            prio as u8
        }
    }

    /// Sets the hardware priority of `system_handler` to `prio`
    ///
    /// *NOTE*: Hardware priority does not exactly match logical priority levels. See
    /// [`NVIC.get_priority`](struct.NVIC.html#method.get_priority) for more details.
    ///
    /// On ARMv6-M, updating a system handler priority requires a read-modify-write operation. On
    /// ARMv7-M, the operation is performed in a single, atomic write operation.
    ///
    /// # Unsafety
    ///
    /// Changing priority levels can break priority-based critical sections (see
    /// [`register::basepri`](../register/basepri/index.html)) and compromise memory safety.
    #[inline]
    pub unsafe fn set_priority(&mut self, system_handler: SystemHandler, prio: u8) {
        let index = system_handler.index();

        #[cfg(not(armv6m))]
        {
            self.shpr[usize::from(index - 4)].write(prio)
        }

        #[cfg(armv6m)]
        {
            self.shpr[usize::from((index - 8) / 4)].modify(|value| {
                let shift = 8 * (index % 4);
                let mask = 0x0000_00ff << shift;
                let prio = u32::from(prio) << shift;

                (value & !mask) | prio
            });
        }
    }

    /// Return the bit position of the exception enable bit in the SHCSR register
    #[inline]
    #[cfg(not(any(armv6m, armv8m_base)))]
    fn shcsr_enable_shift(exception: Exception) -> Option<u32> {
        match exception {
            Exception::MemoryManagement => Some(16),
            Exception::BusFault => Some(17),
            Exception::UsageFault => Some(18),
            #[cfg(armv8m_main)]
            Exception::SecureFault => Some(19),
            _ => None,
        }
    }

    /// Enable the exception
    ///
    /// If the exception is enabled, when the exception is triggered, the exception handler will be executed instead of the
    /// HardFault handler.
    /// This function is only allowed on the following exceptions:
    /// * `MemoryManagement`
    /// * `BusFault`
    /// * `UsageFault`
    /// * `SecureFault` (can only be enabled from Secure state)
    ///
    /// Calling this function with any other exception will do nothing.
    #[inline]
    #[cfg(not(any(armv6m, armv8m_base)))]
    pub fn enable(&mut self, exception: Exception) {
        if let Some(shift) = SCB::shcsr_enable_shift(exception) {
            // The mutable reference to SCB makes sure that only this code is currently modifying
            // the register.
            unsafe { self.shcsr.modify(|value| value | (1 << shift)) }
        }
    }

    /// Disable the exception
    ///
    /// If the exception is disabled, when the exception is triggered, the HardFault handler will be executed instead of the
    /// exception handler.
    /// This function is only allowed on the following exceptions:
    /// * `MemoryManagement`
    /// * `BusFault`
    /// * `UsageFault`
    /// * `SecureFault` (can not be changed from Non-secure state)
    ///
    /// Calling this function with any other exception will do nothing.
    #[inline]
    #[cfg(not(any(armv6m, armv8m_base)))]
    pub fn disable(&mut self, exception: Exception) {
        if let Some(shift) = SCB::shcsr_enable_shift(exception) {
            // The mutable reference to SCB makes sure that only this code is currently modifying
            // the register.
            unsafe { self.shcsr.modify(|value| value & !(1 << shift)) }
        }
    }

    /// Check if an exception is enabled
    ///
    /// This function is only allowed on the following exception:
    /// * `MemoryManagement`
    /// * `BusFault`
    /// * `UsageFault`
    /// * `SecureFault` (can not be read from Non-secure state)
    ///
    /// Calling this function with any other exception will read `false`.
    #[inline]
    #[cfg(not(any(armv6m, armv8m_base)))]
    pub fn is_enabled(&self, exception: Exception) -> bool {
        if let Some(shift) = SCB::shcsr_enable_shift(exception) {
            (self.shcsr.read() & (1 << shift)) > 0
        } else {
            false
        }
    }
}
