use crate::mem::{phys_to_virt, PhysAddr, PAGE_SIZE_4K};
use crate::time::{busy_wait, Duration};

const START_PAGE_IDX: u8 = 6;
const START_PAGE_PADDR: PhysAddr = PhysAddr::from(START_PAGE_IDX as usize * PAGE_SIZE_4K);

core::arch::global_asm!(
    include_str!("ap_start.S"),
    start_page_paddr = const START_PAGE_PADDR.as_usize(),
);

unsafe fn setup_startup_page(stack_top: PhysAddr) {
    extern "C" {
        fn ap_entry32();
        fn ap_start();
        fn ap_end();
    }
    const U64_PER_PAGE: usize = PAGE_SIZE_4K / 8;

    let start_page_ptr = phys_to_virt(START_PAGE_PADDR).as_mut_ptr() as *mut u64;
    let start_page = core::slice::from_raw_parts_mut(start_page_ptr, U64_PER_PAGE);
    core::ptr::copy_nonoverlapping(
        ap_start as *const u64,
        start_page_ptr,
        (ap_end as usize - ap_start as usize) / 8,
    );
    start_page[U64_PER_PAGE - 2] = stack_top.as_usize() as u64; // stack_top
    start_page[U64_PER_PAGE - 1] = ap_entry32 as usize as _; // entry
}

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(apic_id: usize, stack_top: PhysAddr) {
    unsafe { setup_startup_page(stack_top) };

    let apic_id = super::apic::raw_apic_id(apic_id as u8);
    let lapic = super::apic::local_apic();

    // INIT-SIPI-SIPI Sequence
    // Ref: Intel SDM Vol 3C, Section 8.4.4, MP Initialization Example
    unsafe { lapic.send_init_ipi(apic_id) };
    busy_wait(Duration::from_millis(10)); // 10ms
    unsafe { lapic.send_sipi(START_PAGE_IDX, apic_id) };
    busy_wait(Duration::from_micros(200)); // 200us
    unsafe { lapic.send_sipi(START_PAGE_IDX, apic_id) };
}
