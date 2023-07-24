use crate::lapic::lapic_msr::*;
use crate::lapic::{LocalApic, LocalApicMode};
use raw_cpuid::CpuId;

/// The builder pattern for configuring the local APIC.
#[derive(Debug, Default)]
pub struct LocalApicBuilder {
    timer_vector: Option<usize>,
    error_vector: Option<usize>,
    spurious_vector: Option<usize>,

    timer_mode: Option<TimerMode>,
    timer_divide: Option<TimerDivide>,
    timer_initial: Option<u32>,

    ipi_destination_mode: Option<IpiDestMode>,

    xapic_base: Option<u64>,
}

impl LocalApicBuilder {
    /// Returns a new local APIC builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the interrupt index of timer interrupts.
    ///
    /// This field is required.
    pub fn timer_vector(&mut self, index: usize) -> &mut Self {
        self.timer_vector = Some(index);
        self
    }

    /// Sets the interrupt index for internal APIC errors.
    ///
    /// This field is required.
    pub fn error_vector(&mut self, index: usize) -> &mut Self {
        self.error_vector = Some(index);
        self
    }

    /// Sets the interrupt index for spurious interrupts.
    ///
    /// This field is required.
    pub fn spurious_vector(&mut self, index: usize) -> &mut Self {
        self.spurious_vector = Some(index);
        self
    }

    /// Sets the timer mode.
    ///
    /// Default: Periodic.
    pub fn timer_mode(&mut self, mode: TimerMode) -> &mut Self {
        self.timer_mode = Some(mode);
        self
    }

    /// Sets the timer divide configuration.
    ///
    /// Default: Div256.
    pub fn timer_divide(&mut self, divide: TimerDivide) -> &mut Self {
        self.timer_divide = Some(divide);
        self
    }

    /// Sets the timer initial count.
    ///
    /// Default: 10_000_000.
    pub fn timer_initial(&mut self, initial: u32) -> &mut Self {
        self.timer_initial = Some(initial);
        self
    }

    /// Sets the IPI destination mode.
    ///
    /// Default: Physical.
    pub fn ipi_destination_mode(&mut self, mode: IpiDestMode) -> &mut Self {
        self.ipi_destination_mode = Some(mode);
        self
    }

    /// Set the base address for XApic.
    ///
    /// This field is required only if xapic is to be used.
    pub fn set_xapic_base(&mut self, value: u64) -> &mut Self {
        self.xapic_base = Some(value);
        self
    }

    /// Builds a new `LocalApic`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// 1. the CPU does not support the x2apic interrupt architecture, or
    /// 2. any of the required fields are empty.
    pub fn build(&mut self) -> Result<LocalApic, &'static str> {
        let mode = if cpu_has_x2apic() {
            LocalApicMode::X2Apic
        } else {
            if self.xapic_base.is_none() {
                return Err("LocalApicBuilder: XApic base is required.");
            }

            LocalApicMode::XApic {
                xapic_base: self.xapic_base.unwrap(),
            }
        };

        if self.timer_vector.is_none()
            || self.error_vector.is_none()
            || self.spurious_vector.is_none()
        {
            return Err("LocalApicBuilder: required field(s) empty");
        }

        Ok(LocalApic {
            timer_vector: self.timer_vector.unwrap(),
            error_vector: self.error_vector.unwrap(),
            spurious_vector: self.spurious_vector.unwrap(),
            timer_mode: self.timer_mode.unwrap_or(TimerMode::Periodic),
            timer_divide: self.timer_divide.unwrap_or(TimerDivide::Div256),
            timer_initial: self.timer_initial.unwrap_or(10_000_000),
            ipi_destination_mode: self
                .ipi_destination_mode
                .unwrap_or(IpiDestMode::Physical),
            regs: LocalApicRegisters::new(mode),
            mode: mode,
        })
    }
}

fn cpu_has_x2apic() -> bool {
    let cpuid = CpuId::new();

    match cpuid.get_feature_info() {
        Some(finfo) => finfo.has_x2apic(),
        None => false,
    }
}

/// Get the XAPIC Base address.
/// This function reads from `IA32_APIC_BASE`.
pub unsafe fn xapic_base() -> u64 {
    x86_64::registers::model_specific::Msr::new(IA32_APIC_BASE).read()
        & 0xFFFFFF000 as u64
}
