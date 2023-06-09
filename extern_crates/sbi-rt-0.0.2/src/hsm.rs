//! Chapter 9. Hart State Management Extension (EID #0x48534D "HSM")

use crate::binary::{sbi_call_0, sbi_call_1, sbi_call_3, SbiRet};

use sbi_spec::hsm::{EID_HSM, HART_GET_STATUS, HART_START, HART_STOP, HART_SUSPEND};

/// Start executing the given hart at specified address in supervisor-mode.
///
/// This function requests the SBI implementation to start executing the given hart at specified address in supervisor-mode.
///
/// This call is asynchronous - more specifically, the `sbi_hart_start()` may return before target hart
/// starts executing as long as the SBI implemenation is capable of ensuring the return code is accurate.
///
/// It is recommended that if the SBI implementation is a platform runtime firmware executing in machine-mode (M-mode)
/// then it MUST configure PMP and other the M-mode state before executing in supervisor-mode.
///
/// # Parameters
///
/// - The `hartid` parameter specifies the target hart which is to be started.
/// - The `start_addr` parameter points to a runtime-specified physical address, where the hart can start executing in supervisor-mode.
/// - The `opaque` parameter is a `usize` value which will be set in the `a1` register when the hart starts executing at `start_addr`.
///
/// # Behavior
///
/// The target hart jumps to supervisor mode at address specified by `start_addr` with following values in specific registers.
///
/// | Register Name | Register Value
/// |:--------------|:--------------
/// | `satp`        | 0
/// | `sstatus.SIE` | 0
/// | a0            | hartid
/// | a1            | `opaque` parameter
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code               | Description
/// |:--------------------------|:----------------------------------------------
/// | SBI_SUCCESS               | Hart was previously in stopped state. It will start executing from `start_addr`.
/// | SBI_ERR_INVALID_ADDRESS   | `start_addr` is not valid possibly due to following reasons: 1. It is not a valid physical address. 2. The address is prohibited by PMP to run in supervisor mode.
/// | SBI_ERR_INVALID_PARAM     | `hartid` is not a valid hartid as corresponding hart cannot started in supervisor mode.
/// | SBI_ERR_ALREADY_AVAILABLE | The given hartid is already started.
/// | SBI_ERR_FAILED            | The start request failed for unknown reasons.
///
/// This function is defined in RISC-V SBI Specification chapter 9.1.
#[inline]
pub fn hart_start(hartid: usize, start_addr: usize, opaque: usize) -> SbiRet {
    sbi_call_3(EID_HSM, HART_START, hartid, start_addr, opaque)
}

/// Stop executing the calling hart in supervisor-mode.
///
/// This function requests the SBI implementation to stop executing the calling hart in
/// supervisor-mode and return its ownership to the SBI implementation.
///
/// This call is not expected to return under normal conditions.
/// The `sbi_hart_stop()` must be called with the supervisor-mode interrupts disabled.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Error code  | Description
/// |:------------|:------------
/// | `SbiRet::failed()` | Failed to stop execution of the current hart
///
/// This function is defined in RISC-V SBI Specification chapter 9.2.
#[inline]
pub fn hart_stop() -> SbiRet {
    sbi_call_0(EID_HSM, HART_STOP)
}

/// Get the current status (or HSM state id) of the given hart.
///
/// The harts may transition HSM states at any time due to any concurrent `sbi_hart_start()`
/// or `sbi_hart_stop()` calls, the return value from this function may not represent the actual state
/// of the hart at the time of return value verification.
///
/// # Parameters
///
/// The `hartid` parameter specifies the target hart which status is required.
///
/// # Return value
///
/// The possible status values returned in `SbiRet.value` are shown in the table below:
///
/// | Name          | Value | Description
/// |:--------------|:------|:-------------------------
/// | STARTED       |   0   | Hart Started
/// | STOPPED       |   1   | Hart Stopped
/// | START_PENDING |   2   | Hart start request pending
/// | STOP_PENDING  |   3   | Hart stop request pending
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Error code  | Description
/// |:------------|:------------
/// | `SbiRet::invalid_parameter()` | The given `hartid` is not valid
///
/// This function is defined in RISC-V SBI Specification chapter 9.3.
#[inline]
pub fn hart_get_status(hartid: usize) -> SbiRet {
    sbi_call_1(EID_HSM, HART_GET_STATUS, hartid)
}

/// Put the calling hart into suspend or platform specific lower power states.
///
/// This function requests the SBI implementation to put the calling hart in a platform specfic suspend
/// (or low power) state specified by the `suspend_type` parameter.
///
/// The hart will automatically come out of suspended state and resume normal execution
/// when it recieves an interrupt or platform specific hardware event.
///
/// # Suspend behavior
///
/// The platform specific suspend states for a hart can be either retentive or non-rententive in nature.
///
/// A retentive suspend state will preserve hart register and CSR values for all privilege modes,
/// whereas a non-retentive suspend state will not preserve hart register and CSR values.
///
/// # Resuming
///
/// Resuming from a retentive suspend state is straight forward and the supervisor-mode software
/// will see SBI suspend call return without any failures.
///
/// Resuming from a non-retentive suspend state is relatively more involved and requires software
/// to restore various hart registers and CSRs for all privilege modes.
/// Upon resuming from non-retentive suspend state, the hart will jump to supervisor-mode at address
/// specified by `resume_addr` with specific registers values described in the table below:
///
/// | Register Name | Register Value
/// |:--------------|:--------------
/// | `satp`        | 0
/// | `sstatus.SIE` | 0
/// | a0            | hartid
/// | a1            | `opaque` parameter
///
/// # Parameters
///
/// The `suspend_type` parameter is 32 bits wide and the possible values are shown in the table below:
///
/// | Value                   | Description
/// |:------------------------|:--------------
/// | 0x00000000              | Default retentive suspend
/// | 0x00000001 - 0x0FFFFFFF | _Reserved for future use_
/// | 0x10000000 - 0x7FFFFFFF | Platform specific retentive suspend
/// | 0x80000000              | Default non-retentive suspend
/// | 0x80000001 - 0x8FFFFFFF | _Reserved for future use_
/// | 0x90000000 - 0xFFFFFFFF | Platform specific non-retentive suspend
/// | > 0xFFFFFFFF            | _Reserved_
///
/// The `resume_addr` parameter points to a runtime-specified physical address,
/// where the hart can resume execution in supervisor-mode after a non-retentive
/// suspend.
///
/// The `opaque` parameter is a XLEN-bit value which will be set in the `a1`
/// register when the hart resumes exectution at `resume_addr` after a
/// non-retentive suspend.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Error code                  | Description
/// |:----------------------------|:------------
/// | `SbiRet::ok()`              | Hart has suspended and resumed back successfully from a retentive suspend state.
/// | `SbiRet::invalid_param()`   | `suspend_type` is not valid.
/// | `SbiRet::not_supported()`   | `suspend_type` is valid but not implemented.
/// | `SbiRet::invalid_address()` | `resume_addr` is not valid possibly due to following reasons: it is not a valid physical address, or the address is prohibited by PMP to run in supervisor mode.
/// | `SbiRet::failed()`          | The suspend request failed for unknown reasons.
///
/// This function is defined in RISC-V SBI Specification chapter 9.4.
#[inline]
pub fn hart_suspend<T>(suspend_type: T, resume_addr: usize, opaque: usize) -> SbiRet
where
    T: SuspendType,
{
    sbi_call_3(
        EID_HSM,
        HART_SUSPEND,
        suspend_type.raw() as _,
        resume_addr,
        opaque,
    )
}

/// A valid suspend type for hart state monitor
pub trait SuspendType {
    /// Get a raw value to pass to SBI environment
    fn raw(&self) -> u32;
}

#[cfg(feature = "integer-impls")]
impl SuspendType for u32 {
    #[inline]
    fn raw(&self) -> u32 {
        *self
    }
}

macro_rules! define_suspend_type {
    ($($struct:ident($value:expr) #[$doc:meta])*) => {
        $(
            #[derive(Clone, Copy, Debug)]
            #[$doc]
            pub struct $struct;
            impl SuspendType for $struct {
                #[inline]
                fn raw(&self) -> u32 {
                    $value
                }
            }
        )*
    };
}

define_suspend_type! {
    Retentive(sbi_spec::hsm::HART_SUSPEND_TYPE_RETENTIVE) /// Default retentive hart suspension
    NonRetentive(sbi_spec::hsm::HART_SUSPEND_TYPE_NON_RETENTIVE) /// Default non-retentive hart suspension
}
