use core::fmt;

use lazyinit::LazyInit;
use x86_64::instructions::tables::{lgdt, load_tss};
use x86_64::registers::segmentation::{CS, Segment, SegmentSelector};
use x86_64::structures::gdt::{Descriptor, DescriptorFlags};
use x86_64::structures::{DescriptorTablePointer, tss::TaskStateSegment};
use x86_64::{PrivilegeLevel, addr::VirtAddr};

#[unsafe(no_mangle)]
#[percpu::def_percpu]
static TSS: TaskStateSegment = TaskStateSegment::new();

#[percpu::def_percpu]
static GDT: LazyInit<GdtStruct> = LazyInit::new();

/// A wrapper of the Global Descriptor Table (GDT) with maximum 16 entries.
#[repr(align(16))]
pub struct GdtStruct {
    table: [u64; 16],
}

impl GdtStruct {
    /// Kernel code segment for 32-bit mode.
    pub const KCODE32_SELECTOR: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);
    /// Kernel code segment for 64-bit mode.
    pub const KCODE64_SELECTOR: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);
    /// Kernel data segment.
    pub const KDATA_SELECTOR: SegmentSelector = SegmentSelector::new(3, PrivilegeLevel::Ring0);
    /// User code segment for 32-bit mode.
    pub const UCODE32_SELECTOR: SegmentSelector = SegmentSelector::new(4, PrivilegeLevel::Ring3);
    /// User data segment.
    pub const UDATA_SELECTOR: SegmentSelector = SegmentSelector::new(5, PrivilegeLevel::Ring3);
    /// User code segment for 64-bit mode.
    pub const UCODE64_SELECTOR: SegmentSelector = SegmentSelector::new(6, PrivilegeLevel::Ring3);
    /// TSS segment.
    pub const TSS_SELECTOR: SegmentSelector = SegmentSelector::new(7, PrivilegeLevel::Ring0);

    /// Constructs a new GDT struct that filled with the default segment
    /// descriptors, including the given TSS segment.
    pub fn new(tss: &'static TaskStateSegment) -> Self {
        let mut table = [0; 16];
        // first 3 entries are the same as in multiboot.S
        table[1] = DescriptorFlags::KERNEL_CODE32.bits(); // 0x00cf9b000000ffff
        table[2] = DescriptorFlags::KERNEL_CODE64.bits(); // 0x00af9b000000ffff
        table[3] = DescriptorFlags::KERNEL_DATA.bits(); // 0x00cf93000000ffff
        table[4] = DescriptorFlags::USER_CODE32.bits(); // 0x00cffb000000ffff
        table[5] = DescriptorFlags::USER_DATA.bits(); // 0x00cff3000000ffff
        table[6] = DescriptorFlags::USER_CODE64.bits(); // 0x00affb000000ffff
        if let Descriptor::SystemSegment(low, high) = Descriptor::tss_segment(tss) {
            table[7] = low;
            table[8] = high;
        }
        Self { table }
    }

    /// Returns the GDT pointer (base and limit) that can be used in `lgdt`
    /// instruction.
    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (core::mem::size_of_val(&self.table) - 1) as u16,
        }
    }

    /// Loads the GDT into the CPU (executes the `lgdt` instruction), and
    /// updates the code segment register (`CS`).
    ///
    /// # Safety
    ///
    /// This function is unsafe because it manipulates the CPU's privileged
    /// states.
    pub unsafe fn load(&'static self) {
        unsafe {
            lgdt(&self.pointer());
            CS::set_reg(Self::KCODE64_SELECTOR);
        }
    }

    /// Loads the TSS into the CPU (executes the `ltr` instruction).
    ///
    /// # Safety
    ///
    /// This function is unsafe because it manipulates the CPU's privileged
    /// states.
    pub unsafe fn load_tss(&'static self) {
        unsafe {
            load_tss(Self::TSS_SELECTOR);
        }
    }
}

impl fmt::Debug for GdtStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GdtStruct")
            .field("pointer", &self.pointer())
            .field("table", &self.table)
            .finish()
    }
}

/// Initializes the per-CPU TSS and GDT structures and loads them into the
/// current CPU.
pub fn init_gdt() {
    unsafe {
        let gdt = GDT.current_ref_raw();
        gdt.init_once(GdtStruct::new(TSS.current_ref_raw()));
        gdt.load();
        gdt.load_tss();
    }
}

/// Returns the stack pointer for privilege level 0 (RSP0) of the current TSS.
pub fn tss_get_rsp0() -> memory_addr::VirtAddr {
    let tss = unsafe { TSS.current_ref_raw() };
    memory_addr::VirtAddr::from(tss.privilege_stack_table[0].as_u64() as usize)
}

/// Sets the stack pointer for privilege level 0 (RSP0) of the current TSS.
///
/// # Safety
///
/// Must be called after initialization and preemption is disabled.
pub unsafe fn tss_set_rsp0(rsp0: memory_addr::VirtAddr) {
    let tss = unsafe { TSS.current_ref_mut_raw() };
    tss.privilege_stack_table[0] = VirtAddr::new_truncate(rsp0.as_usize() as u64);
}
