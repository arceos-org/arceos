use raw_cpuid::CpuId;

#[cfg(feature = "irq")]
const LAPIC_TICKS_PER_SEC: u64 = 1_000_000_000; // TODO: need to calibrate

#[cfg(feature = "irq")]
static mut NANOS_TO_LAPIC_TICKS_RATIO: ratio::Ratio = ratio::Ratio::zero();

static mut INIT_TICK: u64 = 0;
static mut CPU_FREQ_MHZ: u64 = axconfig::TIMER_FREQUENCY as u64 / 1_000_000;

/// Returns the current clock time in hardware ticks.
pub fn current_ticks() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() - INIT_TICK }
}

/// Converts hardware ticks to nanoseconds.
pub fn ticks_to_nanos(ticks: u64) -> u64 {
    ticks * 1_000 / unsafe { CPU_FREQ_MHZ }
}

/// Converts nanoseconds to hardware ticks.
pub fn nanos_to_ticks(nanos: u64) -> u64 {
    nanos * unsafe { CPU_FREQ_MHZ } / 1_000
}

/// Set a one-shot timer.
///
/// A timer interrupt will be triggered at the given deadline (in nanoseconds).
#[cfg(feature = "irq")]
pub fn set_oneshot_timer(deadline_ns: u64) {
    let lapic = super::apic::local_apic();
    let now_ns = crate::time::current_time_nanos();
    unsafe {
        if now_ns < deadline_ns {
            let apic_ticks = NANOS_TO_LAPIC_TICKS_RATIO.mul_trunc(deadline_ns - now_ns);
            assert!(apic_ticks <= u32::MAX as u64);
            lapic.set_timer_initial(apic_ticks.max(1) as u32);
        } else {
            lapic.set_timer_initial(1);
        }
    }
}

pub(super) fn init_early() {
    if let Some(freq) = CpuId::new()
        .get_processor_frequency_info()
        .map(|info| info.processor_base_frequency())
    {
        if freq > 0 {
            axlog::ax_println!("Got TSC frequency by CPUID: {} MHz", freq);
            unsafe { CPU_FREQ_MHZ = freq as u64 }
        }
    }

    unsafe { INIT_TICK = core::arch::x86_64::_rdtsc() };
}

pub(super) fn init_primary() {
    #[cfg(feature = "irq")]
    unsafe {
        use x2apic::lapic::{TimerDivide, TimerMode};
        let lapic = super::apic::local_apic();
        lapic.set_timer_mode(TimerMode::OneShot);
        lapic.set_timer_divide(TimerDivide::Div256); // indeed it is Div1, the name is confusing.
        lapic.enable_timer();

        // TODO: calibrate with HPET
        NANOS_TO_LAPIC_TICKS_RATIO = ratio::Ratio::new(
            LAPIC_TICKS_PER_SEC as u32,
            crate::time::NANOS_PER_SEC as u32,
        );
    }
}

#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    #[cfg(feature = "irq")]
    unsafe {
        super::apic::local_apic().enable_timer();
    }
}
