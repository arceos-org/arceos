//! The IOAPIC interface.

mod ioapic_regs;
use ioapic_regs::*;

mod irq_entry;
pub use irq_entry::{IrqFlags, IrqMode, RedirectionTableEntry};

/// The IOAPIC structure.
#[derive(Debug)]
pub struct IoApic {
    regs: IoApicRegisters,
}

impl IoApic {
    /// Returns an IOAPIC with the given MMIO address `base_addr`.
    ///
    /// **The given MMIO address must already be mapped.**
    pub unsafe fn new(base_addr: u64) -> Self {
        IoApic {
            regs: IoApicRegisters::new(base_addr),
        }
    }

    /// Initialize the IOAPIC's redirection table entries with the given
    /// interrupt offset.
    ///
    /// Each entry `i` is redirected to `i + offset`.
    pub unsafe fn init(&mut self, offset: u8) {
        let end = self.max_table_entry() + 1;

        for i in 0..end {
            self.regs.set(irq_entry::lo(i), u32::from(i + offset));
            self.regs.write(irq_entry::hi(i), 0);
        }
    }

    /// Returns the IOAPIC ID.
    pub unsafe fn id(&mut self) -> u8 {
        ((self.regs.read(ID) >> 24) & 0xf) as u8
    }

    /// Sets the IOAPIC ID to `id`.
    pub unsafe fn set_id(&mut self, id: u8) {
        self.regs.write(ID, u32::from(id) << 24);
    }

    /// Returns the IOAPIC version.
    pub unsafe fn version(&mut self) -> u8 {
        (self.regs.read(VERSION) & 0xff) as u8
    }

    /// Returns the entry number (starting at zero) of the highest entry in the
    /// redirection table.
    pub unsafe fn max_table_entry(&mut self) -> u8 {
        ((self.regs.read(VERSION) >> 16) & 0xff) as u8
    }

    /// Returns the IOAPIC arbitration ID.
    pub unsafe fn arbitration_id(&mut self) -> u8 {
        ((self.regs.read(ARBITRATION) >> 24) & 0xf) as u8
    }

    /// Sets the IOAPIC arbitration ID to `id`.
    pub unsafe fn set_arbitration_id(&mut self, id: u8) {
        self.regs.write(ARBITRATION, u32::from(id) << 24);
    }

    /// Returns the redirection table entry of `irq`.
    pub unsafe fn table_entry(&mut self, irq: u8) -> RedirectionTableEntry {
        let lo = irq_entry::lo(irq);
        let hi = irq_entry::hi(irq);
        RedirectionTableEntry::from_raw(self.regs.read(lo), self.regs.read(hi))
    }

    /// Configures the redirection table entry of `irq` to `entry`.
    pub unsafe fn set_table_entry(
        &mut self,
        irq: u8,
        entry: RedirectionTableEntry,
    ) {
        let lo = irq_entry::lo(irq);
        let hi = irq_entry::hi(irq);
        let (lo_value, hi_value) = entry.into_raw();
        self.regs.write(lo, lo_value);
        self.regs.write(hi, hi_value);
    }

    /// Enable interrupt number `irq`.
    pub unsafe fn enable_irq(&mut self, irq: u8) {
        self.regs.clear(irq_entry::lo(irq), IrqFlags::MASKED.bits());
    }

    /// Disable interrupt number `irq`.
    pub unsafe fn disable_irq(&mut self, irq: u8) {
        self.regs.set(irq_entry::lo(irq), IrqFlags::MASKED.bits());
    }
}
