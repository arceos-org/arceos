#![allow(dead_code)]

use lazy_init::LazyInit;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;
use x2apic::ioapic::IoApic;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};
use x86_64::instructions::port::Port;

use self::vectors::*;
use crate::mem::phys_to_virt;

pub(super) mod vectors {
    pub const APIC_TIMER_VECTOR: u8 = 0xf0;
    pub const APIC_SPURIOUS_VECTOR: u8 = 0xf1;
    pub const APIC_ERROR_VECTOR: u8 = 0xf2;
}

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 256;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = APIC_TIMER_VECTOR as usize;

const IO_APIC_BASE: PhysAddr = PhysAddr::from(0xFEC0_0000);

static mut LOCAL_APIC: Option<LocalApic> = None;
static mut IS_X2APIC: bool = false;
static IO_APIC: LazyInit<SpinNoIrq<IoApic>> = LazyInit::new();

/// Enables or disables the given IRQ.
#[cfg(feature = "irq")]
pub fn set_enable(vector: usize, enabled: bool) {
    // should not affect LAPIC interrupts
    if vector < APIC_TIMER_VECTOR as _ {
        unsafe {
            if enabled {
                IO_APIC.lock().enable_irq(vector as u8);
            } else {
                IO_APIC.lock().disable_irq(vector as u8);
            }
        }
    }
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
#[cfg(feature = "irq")]
pub fn register_handler(vector: usize, handler: crate::irq::IrqHandler) -> bool {
    crate::irq::register_handler_common(vector, handler)
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
#[cfg(feature = "irq")]
pub fn dispatch_irq(vector: usize) {
    crate::irq::dispatch_irq_common(vector);
    unsafe { local_apic().end_of_interrupt() };
}

pub(super) fn local_apic<'a>() -> &'a mut LocalApic {
    // It's safe as LAPIC is per-cpu.
    unsafe { LOCAL_APIC.as_mut().unwrap() }
}

pub(super) fn raw_apic_id(id_u8: u8) -> u32 {
    if unsafe { IS_X2APIC } {
        id_u8 as u32
    } else {
        (id_u8 as u32) << 24
    }
}

fn cpu_has_x2apic() -> bool {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.has_x2apic(),
        None => false,
    }
}

pub(super) fn init_primary() {
    info!("Initialize Local APIC...");

    unsafe {
        // Disable 8259A interrupt controllers
        Port::<u8>::new(0x21).write(0xff);
        Port::<u8>::new(0xA1).write(0xff);
    }

    let mut builder = LocalApicBuilder::new();
    builder
        .timer_vector(APIC_TIMER_VECTOR as _)
        .error_vector(APIC_ERROR_VECTOR as _)
        .spurious_vector(APIC_SPURIOUS_VECTOR as _);

    if cpu_has_x2apic() {
        info!("Using x2APIC.");
        unsafe { IS_X2APIC = true };
    } else {
        info!("Using xAPIC.");
        let base_vaddr = phys_to_virt(PhysAddr::from(unsafe { xapic_base() } as usize));
        builder.set_xapic_base(base_vaddr.as_usize() as u64);
    }

    let mut lapic = builder.build().unwrap();
    unsafe {
        lapic.enable();
        LOCAL_APIC = Some(lapic);
    }

    info!("Initialize IO APIC...");
    let io_apic = unsafe { IoApic::new(phys_to_virt(IO_APIC_BASE).as_usize() as u64) };
    IO_APIC.init_by(SpinNoIrq::new(io_apic));
}

#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    unsafe { local_apic().enable() };
}
