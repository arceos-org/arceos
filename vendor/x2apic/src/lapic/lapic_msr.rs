#![allow(dead_code)]

use bit::BitIndex;
use bitflags::bitflags;
use core::ops::Range;
use paste;
use x86_64::registers::model_specific::Msr;

use crate::lapic::LocalApicMode;

#[derive(Debug)]
pub enum LocalApicRegister {
    X2apic(Msr),
    XapicOffset(u64),
}

impl LocalApicRegister {
    pub fn new(mode: LocalApicMode, loc: (u32, u32)) -> Self {
        match mode {
            LocalApicMode::XApic { xapic_base } => {
                Self::XapicOffset(xapic_base + loc.1 as u64)
            }
            LocalApicMode::X2Apic => Self::X2apic(Msr::new(loc.0)),
        }
    }

    pub unsafe fn read(&self) -> u32 {
        match self {
            Self::XapicOffset(addr) => {
                core::ptr::read_volatile(*addr as *const u32)
            }
            Self::X2apic(msr) => msr.read() as u32,
        }
    }

    pub unsafe fn read_u64(&self) -> u64 {
        match self {
            Self::XapicOffset(addr) => {
                let lower = core::ptr::read_volatile(*addr as *const u32);
                let upper =
                    core::ptr::read_volatile((*addr + 0x10) as *const u32);
                (lower as u64) | ((upper as u64) << 32)
            }
            Self::X2apic(msr) => msr.read(),
        }
    }

    pub unsafe fn write(&mut self, value: u32) {
        match self {
            LocalApicRegister::XapicOffset(offset) => {
                core::ptr::write_volatile(*offset as *mut u32, value)
            }
            LocalApicRegister::X2apic(msr) => msr.write(value as u64),
        }
    }

    pub unsafe fn write_u64(&mut self, value: u64) {
        match self {
            LocalApicRegister::XapicOffset(offset) => {
                core::ptr::write_volatile(
                    (*offset + 0x10) as *mut u32,
                    (value >> 32) as u32,
                );
                core::ptr::write_volatile(*offset as *mut u32, value as u32);
            }
            LocalApicRegister::X2apic(msr) => msr.write(value),
        }
    }
}

#[derive(Debug)]
pub struct LocalApicRegisters {
    base: Msr,
    id: LocalApicRegister,
    version: LocalApicRegister,
    tpr: LocalApicRegister,
    ppr: LocalApicRegister,
    eoi: LocalApicRegister,
    ldr: LocalApicRegister,
    sivr: LocalApicRegister,
    isr0: LocalApicRegister,
    isr1: LocalApicRegister,
    isr2: LocalApicRegister,
    isr3: LocalApicRegister,
    isr4: LocalApicRegister,
    isr5: LocalApicRegister,
    isr6: LocalApicRegister,
    isr7: LocalApicRegister,
    tmr0: LocalApicRegister,
    tmr1: LocalApicRegister,
    tmr2: LocalApicRegister,
    tmr3: LocalApicRegister,
    tmr4: LocalApicRegister,
    tmr5: LocalApicRegister,
    tmr6: LocalApicRegister,
    tmr7: LocalApicRegister,
    irr0: LocalApicRegister,
    irr1: LocalApicRegister,
    irr2: LocalApicRegister,
    irr3: LocalApicRegister,
    irr4: LocalApicRegister,
    irr5: LocalApicRegister,
    irr6: LocalApicRegister,
    irr7: LocalApicRegister,
    error: LocalApicRegister,
    icr: LocalApicRegister,
    lvt_timer: LocalApicRegister,
    lvt_thermal: LocalApicRegister,
    lvt_perf: LocalApicRegister,
    lvt_lint0: LocalApicRegister,
    lvt_lint1: LocalApicRegister,
    lvt_error: LocalApicRegister,
    ticr: LocalApicRegister,
    tccr: LocalApicRegister,
    tdcr: LocalApicRegister,
    self_ipi: LocalApicRegister,
}

macro_rules! read {
    ($name:ident) => {
        paste::item! {
            pub unsafe fn $name(&self) -> u32 {
                self.$name.read()
            }

            pub unsafe fn [<$name _bit>](&self, bit: usize) -> bool {
                self.$name().bit(bit)
            }

            pub unsafe fn [<$name _bit_range>](
                &self,
                pos: Range<usize>,
            ) -> u32 {
                self.$name().bit_range(pos)
            }
        }
    };
}

macro_rules! write {
    ($name:ident) => {
        paste::item! {
            pub unsafe fn [<write_ $name>](&mut self, value: u32) {
                self.$name.write(value);
            }
        }
    };
}

macro_rules! read_write {
    ($name:ident) => {
        read!($name);
        write!($name);

        paste::item! {
            pub unsafe fn [<set_ $name _bit>](
                &mut self,
                bit: usize,
                val: bool,
            ) {
                let mut reg_val = self.$name();

                reg_val.set_bit(bit, val);

                self.[<write_ $name>](reg_val);
            }

            pub unsafe fn [<set_ $name _bit_range>](
                &mut self,
                pos: Range<usize>,
                val: u32,
            ) {
                let mut reg_val = self.$name();

                reg_val.set_bit_range(pos, val);

                self.[<write_ $name>](reg_val);
            }
        }
    };
}

impl LocalApicRegisters {
    pub fn new(mode: LocalApicMode) -> Self {
        LocalApicRegisters {
            base: Msr::new(IA32_APIC_BASE),
            id: LocalApicRegister::new(mode, ID),
            version: LocalApicRegister::new(mode, VERSION),
            tpr: LocalApicRegister::new(mode, TPR),
            ppr: LocalApicRegister::new(mode, PPR),
            eoi: LocalApicRegister::new(mode, EOI),
            ldr: LocalApicRegister::new(mode, LDR),
            sivr: LocalApicRegister::new(mode, SIVR),
            isr0: LocalApicRegister::new(mode, ISR_0),
            isr1: LocalApicRegister::new(mode, ISR_1),
            isr2: LocalApicRegister::new(mode, ISR_2),
            isr3: LocalApicRegister::new(mode, ISR_3),
            isr4: LocalApicRegister::new(mode, ISR_4),
            isr5: LocalApicRegister::new(mode, ISR_5),
            isr6: LocalApicRegister::new(mode, ISR_6),
            isr7: LocalApicRegister::new(mode, ISR_7),
            tmr0: LocalApicRegister::new(mode, TMR_0),
            tmr1: LocalApicRegister::new(mode, TMR_1),
            tmr2: LocalApicRegister::new(mode, TMR_2),
            tmr3: LocalApicRegister::new(mode, TMR_3),
            tmr4: LocalApicRegister::new(mode, TMR_4),
            tmr5: LocalApicRegister::new(mode, TMR_5),
            tmr6: LocalApicRegister::new(mode, TMR_6),
            tmr7: LocalApicRegister::new(mode, TMR_7),
            irr0: LocalApicRegister::new(mode, IRR_0),
            irr1: LocalApicRegister::new(mode, IRR_1),
            irr2: LocalApicRegister::new(mode, IRR_2),
            irr3: LocalApicRegister::new(mode, IRR_3),
            irr4: LocalApicRegister::new(mode, IRR_4),
            irr5: LocalApicRegister::new(mode, IRR_5),
            irr6: LocalApicRegister::new(mode, IRR_6),
            irr7: LocalApicRegister::new(mode, IRR_7),
            error: LocalApicRegister::new(mode, ERROR),
            icr: LocalApicRegister::new(mode, ICR),
            lvt_timer: LocalApicRegister::new(mode, LVT_TIMER),
            lvt_thermal: LocalApicRegister::new(mode, LVT_THERMAL),
            lvt_perf: LocalApicRegister::new(mode, LVT_PERF),
            lvt_lint0: LocalApicRegister::new(mode, LVT_LINT0),
            lvt_lint1: LocalApicRegister::new(mode, LVT_LINT1),
            lvt_error: LocalApicRegister::new(mode, LVT_ERROR),
            ticr: LocalApicRegister::new(mode, TICR),
            tccr: LocalApicRegister::new(mode, TCCR),
            tdcr: LocalApicRegister::new(mode, TDCR),
            self_ipi: LocalApicRegister::new(mode, SELF_IPI),
        }
    }

    pub unsafe fn base(&self) -> u64 {
        self.base.read()
    }

    pub unsafe fn base_bit(&self, bit: usize) -> bool {
        self.base().bit(bit)
    }

    pub unsafe fn write_base(&mut self, value: u64) {
        self.base.write(value)
    }

    pub unsafe fn set_base_bit(&mut self, bit: usize, value: bool) {
        let mut base = self.base();
        base.set_bit(bit, value);
        self.write_base(base);
    }

    pub unsafe fn icr(&self) -> u64 {
        self.icr.read_u64()
    }

    pub unsafe fn icr_bit(&self, bit: usize) -> bool {
        self.icr().bit(bit)
    }

    pub unsafe fn write_icr(&mut self, value: u64) {
        self.icr.write_u64(value)
    }

    pub unsafe fn set_icr_bit(&mut self, bit: usize, value: bool) {
        let mut icr = self.icr();
        icr.set_bit(bit, value);
        self.write_icr(icr);
    }

    read!(id);
    read!(version);
    read_write!(tpr);
    read!(ppr);
    write!(eoi);
    read_write!(ldr);
    read_write!(sivr);
    read!(isr0);
    read!(isr1);
    read!(isr2);
    read!(isr3);
    read!(isr4);
    read!(isr5);
    read!(isr6);
    read!(isr7);
    read!(tmr0);
    read!(tmr1);
    read!(tmr2);
    read!(tmr3);
    read!(tmr4);
    read!(tmr5);
    read!(tmr6);
    read!(tmr7);
    read!(irr0);
    read!(irr1);
    read!(irr2);
    read!(irr3);
    read!(irr4);
    read!(irr5);
    read!(irr6);
    read!(irr7);
    read!(error);
    read_write!(lvt_timer);
    read_write!(lvt_thermal);
    read_write!(lvt_perf);
    read_write!(lvt_lint0);
    read_write!(lvt_lint1);
    read_write!(lvt_error);
    read_write!(ticr);
    read!(tccr);
    read_write!(tdcr);
    write!(self_ipi);
}

bitflags! {
    /// Error flags in the APIC error status register.
    pub struct ErrorFlags: u8 {
        /// P6 and Pentium only.
        /// Local APIC detected a checksum error during send.
        const SEND_CHECKSUM_ERROR       = 0b0000_0001;
        /// P6 and Pentium only.
        /// Local APIC detected a checksum error during receive.
        const RECEIVE_CHECKSUM_ERROR    = 0b0000_0010;
        /// P6 and Pentium only.
        /// Local APIC sent a message that was not accepted by any APIC.
        const SEND_ACCEPT_ERROR         = 0b0000_0100;
        /// P6 and Pentium only.
        /// Local APIC received a message that was not accepted by any APIC.
        const RECEIVE_ACCEPT_ERROR      = 0b0000_1000;
        /// Local APIC does not support lowest-priority mode.
        const REDIRECTABLE_IPI          = 0b0001_0000;
        /// Local APIC detected an illegal interrupt vector (0-15) during send.
        const SEND_ILLEGAL_VECTOR       = 0b0010_0000;
        /// Local APIC detected an illegal interrupt vector (0-15) during
        /// receive.
        const RECEIVED_ILLEGAL_VECTOR   = 0b0100_0000;
        /// xAPIC mode only.
        /// Local APIC tried to access a reserved register.
        const ILLEGAL_REGISTER_ADDRESS  = 0b1000_0000;
    }
}

/// Local APIC timer modes.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerMode {
    /// Timer only fires once.
    OneShot = 0b00,
    /// Timer fires periodically.
    Periodic = 0b01,
    /// Timer fires at an absolute time.
    TscDeadline = 0b10,
}

impl TimerMode {
    pub(super) fn into_u32(self) -> u32 {
        self as u32
    }
}

/// Local APIC timer divide configurations.
///
/// Defines the APIC timer frequency as the processor frequency divided by a
/// specified value.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TimerDivide {
    /// Divide by 2.
    Div2 = 0b0000,
    /// Divide by 4.
    Div4 = 0b0001,
    /// Divide by 8.
    Div8 = 0b0010,
    /// Divide by 16.
    Div16 = 0b0011,
    /// Divide by 32.
    Div32 = 0b1000,
    /// Divide by 64.
    Div64 = 0b1001,
    /// Divide by 128.
    Div128 = 0b1010,
    /// Divide by 256.
    Div256 = 0b1011,
}

impl TimerDivide {
    pub(super) fn into_u32(self) -> u32 {
        self as u32
    }
}

/// Inter-processor interrupt destination mode.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum IpiDestMode {
    /// Physical destination mode.
    Physical = 0b0,
    /// Logical destination mode.    
    Logical = 0b1,
}

/// Inter-processor interrupt delivery modes.
#[derive(Debug)]
#[repr(u8)]
pub enum IpiDeliveryMode {
    /// Delivers to the processors specified in the vector field.
    Fixed = 0b000,
    /// Same as fixed, except interrupt is delivered to the processor with the
    /// lowest priority.
    LowestPriority = 0b001,
    /// Delivers a system management interrupt to the target processors.
    SystemManagement = 0b010,
    /// Delivers a non-maskable interrupt to the target processors.
    NonMaskable = 0b100,
    /// Delivers an INIT interrupt to the target processor(s).
    Init = 0b101,
    /// Delivers a start-up IPI to the target processor(s).
    StartUp = 0b110,
}

impl IpiDeliveryMode {
    pub(super) fn into_u64(self) -> u64 {
        self as u64
    }
}

/// Specifies the destination when calling `send_ipi_all`.
#[derive(Debug)]
#[repr(u8)]
pub enum IpiAllShorthand {
    /// Send inter-processor interrupt all processors.
    AllIncludingSelf = 0b10,
    /// Send inter-processor interrupt to all processor except this one.
    AllExcludingSelf = 0b11,
}

impl IpiAllShorthand {
    pub(super) fn into_u64(self) -> u64 {
        self as u64
    }
}

// MSR port addresses

pub const IA32_APIC_BASE: u32 = 0x1B;

// format is (X2APIC MSR, XAPIC MMIO offset)
pub const ID: (u32, u32) = (0x802, 0x020);
pub const VERSION: (u32, u32) = (0x803, 0x030);
pub const TPR: (u32, u32) = (0x808, 0x080);
pub const PPR: (u32, u32) = (0x80A, 0x0A0);
pub const EOI: (u32, u32) = (0x80B, 0x0B0);
pub const LDR: (u32, u32) = (0x80D, 0x0D0);
pub const SIVR: (u32, u32) = (0x80F, 0x0F0);

pub const ISR_0: (u32, u32) = (0x810, 0x100);
pub const ISR_1: (u32, u32) = (0x811, 0x110);
pub const ISR_2: (u32, u32) = (0x812, 0x120);
pub const ISR_3: (u32, u32) = (0x813, 0x130);
pub const ISR_4: (u32, u32) = (0x814, 0x140);
pub const ISR_5: (u32, u32) = (0x815, 0x150);
pub const ISR_6: (u32, u32) = (0x816, 0x160);
pub const ISR_7: (u32, u32) = (0x817, 0x170);

pub const TMR_0: (u32, u32) = (0x818, 0x180);
pub const TMR_1: (u32, u32) = (0x819, 0x190);
pub const TMR_2: (u32, u32) = (0x81A, 0x1A0);
pub const TMR_3: (u32, u32) = (0x81B, 0x1B0);
pub const TMR_4: (u32, u32) = (0x81C, 0x1C0);
pub const TMR_5: (u32, u32) = (0x81D, 0x1D0);
pub const TMR_6: (u32, u32) = (0x81E, 0x1E0);
pub const TMR_7: (u32, u32) = (0x81F, 0x1F0);

pub const IRR_0: (u32, u32) = (0x820, 0x200);
pub const IRR_1: (u32, u32) = (0x821, 0x210);
pub const IRR_2: (u32, u32) = (0x822, 0x220);
pub const IRR_3: (u32, u32) = (0x823, 0x230);
pub const IRR_4: (u32, u32) = (0x824, 0x240);
pub const IRR_5: (u32, u32) = (0x825, 0x250);
pub const IRR_6: (u32, u32) = (0x826, 0x260);
pub const IRR_7: (u32, u32) = (0x827, 0x270);

pub const ERROR: (u32, u32) = (0x828, 0x280);
pub const ICR: (u32, u32) = (0x830, 0x300);

pub const LVT_TIMER: (u32, u32) = (0x832, 0x320);
pub const LVT_THERMAL: (u32, u32) = (0x833, 0x330);
pub const LVT_PERF: (u32, u32) = (0x834, 0x340);
pub const LVT_LINT0: (u32, u32) = (0x835, 0x350);
pub const LVT_LINT1: (u32, u32) = (0x836, 0x360);
pub const LVT_ERROR: (u32, u32) = (0x837, 0x370);

pub const TICR: (u32, u32) = (0x838, 0x380);
pub const TCCR: (u32, u32) = (0x839, 0x390);
pub const TDCR: (u32, u32) = (0x83E, 0x3E0);

pub const SELF_IPI: (u32, u32) = (0x83F, 0xFFFF);

// Register bits and bit ranges.

pub const BASE_APIC_ENABLE: usize = 11;
pub const BASE_X2APIC_ENABLE: usize = 10;
pub const BASE_BSP: usize = 8;

pub const VERSION_NR: Range<usize> = 0..8;
pub const VERSION_MAX_LVT_ENTRY: Range<usize> = 16..24;
pub const VERSION_EOI_BCAST_SUPPRESSION: usize = 24;

pub const SIVR_EOI_BCAST_SUPPRESSION: usize = 12;
pub const SIVR_FOCUS_PROCESSOR_CHECKING: usize = 9;
pub const SIVR_APIC_SOFTWARE_ENABLE: usize = 8;
pub const SIVR_VECTOR: Range<usize> = 0..8;

pub const ICR_DESTINATION: Range<usize> = 32..64;
pub const ICR_DEST_SHORTHAND: Range<usize> = 18..20;
pub const ICR_TRIGGER_MODE: usize = 15;
pub const ICR_LEVEL: usize = 14;
pub const ICR_DESTINATION_MODE: usize = 11;
pub const ICR_DELIVERY_MODE: Range<usize> = 8..11;
pub const ICR_VECTOR: Range<usize> = 0..8;

pub const LVT_TIMER_MODE: Range<usize> = 17..19;
pub const LVT_TIMER_MASK: usize = 16;
pub const LVT_TIMER_VECTOR: Range<usize> = 0..8;

pub const LVT_ERROR_VECTOR: Range<usize> = 0..8;

pub const TDCR_DIVIDE_VALUE: Range<usize> = 0..4;
