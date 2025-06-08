//! Dummy implementation of platform-related interfaces defined in [`axplat`].

use axplat::impl_plat_interface;

use axplat::console::ConsoleIf;
use axplat::init::InitIf;
use axplat::irq::{IrqHandler, IrqIf};
use axplat::mem::{MemIf, RawRange};
use axplat::power::PowerIf;
use axplat::time::TimeIf;

struct DummyInit;
struct DummyConsole;
struct DummyMem;
struct DummyTime;
struct DummyPower;
struct DummyIrq;

#[impl_plat_interface]
impl InitIf for DummyInit {
    fn init_early(_cpu_id: usize, _arg: usize) {}

    fn init_early_secondary(_cpu_id: usize) {}

    fn init_later(_cpu_id: usize, _arg: usize) {}

    fn init_later_secondary(_cpu_id: usize) {}
}

#[impl_plat_interface]
impl ConsoleIf for DummyConsole {
    fn write_bytes(_bytes: &[u8]) {
        unimplemented!()
    }

    fn read_bytes(_bytes: &mut [u8]) -> usize {
        unimplemented!()
    }
}

#[impl_plat_interface]
impl MemIf for DummyMem {
    fn phys_ram_ranges() -> &'static [RawRange] {
        &[]
    }

    fn reserved_phys_ram_ranges() -> &'static [RawRange] {
        &[]
    }

    fn mmio_ranges() -> &'static [RawRange] {
        &[]
    }
}

#[impl_plat_interface]
impl TimeIf for DummyTime {
    fn current_ticks() -> u64 {
        0
    }

    fn ticks_to_nanos(ticks: u64) -> u64 {
        ticks
    }

    fn nanos_to_ticks(nanos: u64) -> u64 {
        nanos
    }

    fn epochoffset_nanos() -> u64 {
        0
    }

    fn set_oneshot_timer(_deadline_ns: u64) {}
}

#[impl_plat_interface]
impl PowerIf for DummyPower {
    fn cpu_boot(_cpu_id: usize, _stack_top_paddr: usize) {}

    fn system_off() -> ! {
        unimplemented!()
    }
}

#[impl_plat_interface]
impl IrqIf for DummyIrq {
    fn set_enable(_irq: usize, _enabled: bool) {}

    fn register(_irq: usize, _handler: IrqHandler) -> bool {
        false
    }

    fn unregister(_irq: usize) -> Option<IrqHandler> {
        None
    }

    fn handle(_irq: usize) {}
}
