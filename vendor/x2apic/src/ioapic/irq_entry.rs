use super::TABLE_BASE;
use bitflags::bitflags;
use core::convert::{TryFrom, TryInto};
use core::fmt;

pub const IRQ_MODE_MASK: u32 = 0x0000_0700;

/// IOAPIC interrupt modes.
#[derive(Debug)]
#[repr(u8)]
pub enum IrqMode {
    /// Asserts the INTR signal on all allowed processors.
    Fixed = 0b000,
    /// Asserts the INTR signal on the lowest priority processor allowed.
    LowestPriority = 0b001,
    /// System management interrupt.
    /// Requires edge-triggering.
    SystemManagement = 0b010,
    /// Asserts the NMI signal on all allowed processors.
    /// Requires edge-triggering.
    NonMaskable = 0b100,
    /// Asserts the INIT signal on all allowed processors.
    /// Requires edge-triggering.
    Init = 0b101,
    /// Asserts the INTR signal as a signal that originated in an
    /// externally-connected interrupt controller.
    /// Requires edge-triggering.
    External = 0b111,
}

impl IrqMode {
    pub(super) fn as_u32(self) -> u32 {
        (self as u32) << 8
    }
}

impl TryFrom<u32> for IrqMode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match (value & IRQ_MODE_MASK) >> 8 {
            0b000 => Ok(IrqMode::Fixed),
            0b001 => Ok(IrqMode::LowestPriority),
            0b010 => Ok(IrqMode::SystemManagement),
            0b100 => Ok(IrqMode::NonMaskable),
            0b101 => Ok(IrqMode::Init),
            0b111 => Ok(IrqMode::External),
            other => Err(other),
        }
    }
}

bitflags! {
    /// Redirection table entry flags.
    pub struct IrqFlags: u32 {
        /// Logical destination mode (vs physical)
        const LOGICAL_DEST = 1 << 11;
        /// Delivery status: send pending (vs idle, readonly)
        const SEND_PENDING = 1 << 12;
        /// Low-polarity interrupt signal (vs high-polarity)
        const LOW_ACTIVE = 1 << 13;
        /// Remote IRR (readonly)
        const REMOTE_IRR = 1 << 14;
        /// Level-triggered interrupt (vs edge-triggered)
        const LEVEL_TRIGGERED = 1 << 15;
        /// Masked interrupt (vs unmasked)
        const MASKED = 1 << 16;
    }
}

/// Redirection table entry.
#[derive(Default)]
pub struct RedirectionTableEntry {
    low: u32,
    high: u32,
}

impl RedirectionTableEntry {
    pub(crate) fn from_raw(low: u32, high: u32) -> Self {
        Self { low, high }
    }

    pub(crate) fn into_raw(self) -> (u32, u32) {
        (self.low, self.high)
    }

    /// Returns the interrupt vector.
    pub fn vector(&self) -> u8 {
        (self.low & 0xff) as u8
    }

    /// Sets the interrupt vector to `vector`.
    pub fn set_vector(&mut self, vector: u8) {
        self.low = self.low & !0xff | vector as u32
    }

    /// Returns the interrupt delivery mode.
    pub fn mode(&self) -> IrqMode {
        self.low.try_into().unwrap()
    }

    /// Sets the interrupt delivery mode to `mode`.
    pub fn set_mode(&mut self, mode: IrqMode) {
        self.low = self.low & !IRQ_MODE_MASK | mode.as_u32()
    }

    /// Returns the redirection table entry flags.
    pub fn flags(&self) -> IrqFlags {
        IrqFlags::from_bits_truncate(self.low)
    }

    /// Sets the redirection table entry flags to `flags`.
    pub fn set_flags(&mut self, flags: IrqFlags) {
        let ro_flags = IrqFlags::SEND_PENDING | IrqFlags::REMOTE_IRR;
        self.low = self.low & !(IrqFlags::all() - ro_flags).bits()
            | (flags - ro_flags).bits()
    }

    /// Returns the destination field.
    pub fn dest(&self) -> u8 {
        (self.high >> 24) as u8
    }

    /// Sets the destination field to `dest`.
    pub fn set_dest(&mut self, dest: u8) {
        self.high = (dest as u32) << 24;
    }
}

// Gets the lower segment selector for `irq`
pub fn lo(irq: u8) -> u32 {
    TABLE_BASE + (2 * u32::from(irq))
}

// Gets the upper segment selector for `irq`
pub fn hi(irq: u8) -> u32 {
    lo(irq) + 1
}

impl fmt::Debug for RedirectionTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RedirectionTableEntry")
            .field("vector", &self.vector())
            .field("mode", &self.mode())
            .field("flags", &self.flags())
            .field("dest", &self.dest())
            .finish()
    }
}
