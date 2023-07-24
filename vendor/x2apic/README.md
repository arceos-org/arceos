# x2apic-rs

A Rust interface to the x2apic interrupt architecture.

This crate is in its early stages and has only been tested in QEMU; code
contributions and bug reports are welcome.

It will use x2APIC mode if supported by the CPU, otherwise it
falls back to xAPIC mode.

## Usage

The local APIC is initialized like so:

```rust
use x2apic::lapic::{LocalApic, LocalApicBuilder, xapic_base};

let apic_physical_address: u64 = unsafe { xapic_base() };
let apic_virtual_address: u64 = <convert from physical address>

let lapic = LocalApicBuilder::new()
    .timer_vector(timer_index)
    .error_vector(error_index)
    .spurious_vector(spurious_index)
    .set_xapic_base(apic_virtual_address)
    .build()
    .unwrap_or_else(|err| panic!("{}", err));

unsafe {
    lapic.enable();
}
```

This initializes and enables the local APIC timer with a default configuration.
The timer can be configured with the builder or directly on the APIC.

The IOAPIC is initialized like so:

```rust
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};

// !!! Map the IOAPIC's MMIO address `addr` here !!!

unsafe {
    let ioapic = IoApic::new(addr);

    ioapic.init(irq_offset);

    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(IrqMode::Fixed);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
    entry.set_dest(dest); // CPU(s)
    ioapic.set_table_entry(irq_number, entry);

    ioapic.enable_irq(irq_number);
}
```

Please refer to the documentation for more details.
