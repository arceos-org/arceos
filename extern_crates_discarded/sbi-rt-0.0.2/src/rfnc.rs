//! Chapter 8. RFENCE Extension (EID #0x52464E43 "RFNC")

use crate::binary::{sbi_call_2, sbi_call_4, sbi_call_5, SbiRet};

use sbi_spec::rfnc::{
    EID_RFNC, REMOTE_FENCE_I, REMOTE_HFENCE_GVMA, REMOTE_HFENCE_GVMA_VMID, REMOTE_HFENCE_VVMA,
    REMOTE_HFENCE_VVMA_ASID, REMOTE_SFENCE_VMA, REMOTE_SFENCE_VMA_ASID,
};

/// Execute `FENCE.I` instruction on remote harts.
///
/// # Return value
///
/// Returns `SbiRet::ok()` when remote fence was sent to all the targeted harts successfully.
///
/// This function is defined in RISC-V SBI Specification chapter 8.1.
#[inline]
pub fn remote_fence_i(hart_mask: usize, hart_mask_base: usize) -> SbiRet {
    sbi_call_2(EID_RFNC, REMOTE_FENCE_I, hart_mask, hart_mask_base)
}

/// Execute `SFENCE.VMA` instructions for all address spaces on remote harts.
///
/// This function instructs the remote harts to execute one or more `SFENCE.VMA` instructions,
/// covering the range of virtual addresses between `start_addr` and `size`.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code                 | Description
/// |:----------------------------|:----------------------------------------------
/// | `SbiRet::ok()`              | Remote fence was sent to all the targeted harts successfully.
/// | `SbiRet::invalid_address()` | `start_addr` or `size` is not valid.
///
/// This function is defined in RISC-V SBI Specification chapter 8.2.
#[inline]
pub fn remote_sfence_vma(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
) -> SbiRet {
    sbi_call_4(
        EID_RFNC,
        REMOTE_SFENCE_VMA,
        hart_mask,
        hart_mask_base,
        start_addr,
        size,
    )
}

/// Execute address space based `SFENCE.VMA` instructions on remote harts.
///
/// This function instructs the remote harts to execute one or more `SFENCE.VMA` instructions,
/// covering the range of virtual addresses between `start_addr` and `size`.
/// This covers only the given address space by `asid`.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code                 | Description
/// |:----------------------------|:----------------------------------------------
/// | `SbiRet::ok()`              | Remote fence was sent to all the targeted harts successfully.
/// | `SbiRet::invalid_address()` | `start_addr` or `size` is not valid.
///
/// This function is defined in RISC-V SBI Specification chapter 8.3.
#[inline]
pub fn remote_sfence_vma_asid(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
    asid: usize,
) -> SbiRet {
    sbi_call_5(
        EID_RFNC,
        REMOTE_SFENCE_VMA_ASID,
        hart_mask,
        hart_mask_base,
        start_addr,
        size,
        asid,
    )
}

/// Execute virtual machine id based `HFENCE.GVMA` instructions on remote harts.
///
/// This function instructs the remote harts to execute one or more `HFENCE.GVMA`
/// instructions, covering the range of guest physical addresses between `start_addr`
/// and `size` only for the given virtual machine by `vmid`.
///
/// This function call is only valid for harts implementing hypervisor extension.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code                 | Description
/// |:----------------------------|:----------------------------------------------
/// | `SbiRet::ok()`              | Remote fence was sent to all the targeted harts successfully.
/// | `SbiRet::not_supported()`   | This function is not supported as it is not implemented or one of the target hart doesn’t support hypervisor extension.
/// | `SbiRet::invalid_address()` | `start_addr` or `size` is not valid.
///
/// This function is defined in RISC-V SBI Specification chapter 8.4.
#[inline]
pub fn remote_hfence_gvma_vmid(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
    vmid: usize,
) -> SbiRet {
    sbi_call_5(
        EID_RFNC,
        REMOTE_HFENCE_GVMA_VMID,
        hart_mask,
        hart_mask_base,
        start_addr,
        size,
        vmid,
    )
}

/// Execute `HFENCE.GVMA` instructions for all virtual machines on remote harts.
///
/// This function instructs the remote harts to execute one or more `HFENCE.GVMA` instructions,
/// covering the range of guest physical addresses between `start_addr` and `size`
/// for all the guests.
///
/// This function call is only valid for harts implementing hypervisor extension.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code                 | Description
/// |:----------------------------|:----------------------------------------------
/// | `SbiRet::ok()`              | Remote fence was sent to all the targeted harts successfully.
/// | `SbiRet::not_supported()`   | This function is not supported as it is not implemented or one of the target hart does not support hypervisor extension.
/// | `SbiRet::invalid_address()` | `start_addr` or `size` is not valid.
///
/// This function is defined in RISC-V SBI Specification chapter 8.5.
#[inline]
pub fn remote_hfence_gvma(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
) -> SbiRet {
    sbi_call_4(
        EID_RFNC,
        REMOTE_HFENCE_GVMA,
        hart_mask,
        hart_mask_base,
        start_addr,
        size,
    )
}

/// Execute address space based `HFENCE.VVMA` for current virtual machine on remote harts.
///
/// This function instructs the remote harts to execute one or more `HFENCE.VVMA` instructions,
/// covering the range of guest virtual addresses between `start_addr` and `size` for the given
/// address space by `asid` and current virtual machine (by `vmid` in `hgatp` CSR)
/// of calling hart.
///
/// This function call is only valid for harts implementing hypervisor extension.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code                 | Description
/// |:----------------------------|:----------------------------------------------
/// | `SbiRet::ok()`              | Remote fence was sent to all the targeted harts successfully.
/// | `SbiRet::not_supported()`   | This function is not supported as it is not implemented or one of the target hart does not support hypervisor extension.
/// | `SbiRet::invalid_address()` | `start_addr` or `size` is not valid.
///
/// This function is defined in RISC-V SBI Specification chapter 8.6.
#[inline]
pub fn remote_hfence_vvma_asid(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
    asid: usize,
) -> SbiRet {
    sbi_call_5(
        EID_RFNC,
        REMOTE_HFENCE_VVMA_ASID,
        hart_mask,
        hart_mask_base,
        start_addr,
        size,
        asid,
    )
}

/// Execute `HFENCE.VVMA` for all address spaces in current virtual machine on remote harts.
///
/// This function instructs the remote harts to execute one or more `HFENCE.VVMA` instructions,
/// covering the range of guest virtual addresses between `start_addr` and `size`
/// for current virtual machine (by `vmid` in `hgatp` CSR) of calling hart.
///
/// This function call is only valid for harts implementing hypervisor extension.
///
/// # Return value
///
/// The possible return error codes returned in `SbiRet.error` are shown in the table below:
///
/// | Return code                 | Description
/// |:----------------------------|:----------------------------------------------
/// | `SbiRet::ok()`              | Remote fence was sent to all the targeted harts successfully.
/// | `SbiRet::not_supported()`   | This function is not supported as it is not implemented or one of the target hart doesn’t support hypervisor extension.
/// | `SbiRet::invalid_address()` | `start_addr` or `size` is not valid.
///
/// This function is defined in RISC-V SBI Specification chapter 8.7.
#[inline]
pub fn remote_hfence_vvma(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
) -> SbiRet {
    sbi_call_4(
        EID_RFNC,
        REMOTE_HFENCE_VVMA,
        hart_mask,
        hart_mask_base,
        start_addr,
        size,
    )
}
