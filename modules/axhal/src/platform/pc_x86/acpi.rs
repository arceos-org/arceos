extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use core::alloc::{AllocError, Layout};
use core::ptr::NonNull;

use acpi::{AcpiTables, PhysicalMapping};
use aml::{AmlContext, AmlName, DebugVerbosity};
use aml::pci_routing::{IrqDescriptor, PciRoutingTable, Pin};

use axalloc::global_allocator;
use lazy_init::LazyInit;
use memory_addr::PhysAddr;
use crate::mem::phys_to_virt;

use crate::arch::irq_to_vector;

#[derive(Clone)]
struct LocalAcpiHandler;

impl acpi::AcpiHandler for LocalAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let vaddr = phys_to_virt(PhysAddr::from(physical_address)).as_usize();
        PhysicalMapping::new(
            physical_address.clone(),
            NonNull::new_unchecked(vaddr as *mut i32 as *mut _),
            size.clone(),
            size.clone(),
            self.clone(),
        )
    }
    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

struct LocalAmlHandler;

impl aml::Handler for LocalAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_ptr();
        unsafe { vaddr.read_volatile() }
    }

    fn read_u16(&self, address: usize) -> u16 {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_ptr() as *const u16;
        unsafe { vaddr.read_volatile() }
    }

    fn read_u32(&self, address: usize) -> u32 {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_ptr() as *const u32;
        unsafe { vaddr.read_volatile() }
    }

    fn read_u64(&self, address: usize) -> u64 {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_ptr() as *const u64;
        unsafe { vaddr.read_volatile() }
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_mut_ptr();
        unsafe { vaddr.write_volatile(value) }
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_mut_ptr() as *mut u16;
        unsafe { vaddr.write_volatile(value) }
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_mut_ptr() as *mut u32;
        unsafe { vaddr.write_volatile(value) }
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        let vaddr = phys_to_virt(PhysAddr::from(address)).as_mut_ptr() as *mut u64;
        unsafe { vaddr.write_volatile(value) }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { x86::io::inb(port) }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { x86::io::inw(port) }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { x86::io::inl(port) }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe {
            x86::io::outb(port, value);
        }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe {
            x86::io::outw(port, value);
        }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe {
            x86::io::outl(port, value);
        }
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        let paddr = unsafe {
            ACPI.get_pci_config_regions_addr(segment, bus, device, function)
                .unwrap()
        };
        let vaddr = phys_to_virt(PhysAddr::from(paddr as usize)).as_usize() as *mut u8;
        let address = unsafe { vaddr.add(offset as usize) };
        return unsafe { address.read_volatile() };
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        let paddr = unsafe {
            ACPI.get_pci_config_regions_addr(segment, bus, device, function)
                .unwrap()
        };
        let vaddr = phys_to_virt(PhysAddr::from(paddr as usize)).as_usize() as *mut u16;
        let address = unsafe { vaddr.add(offset as usize) };
        return unsafe { address.read_volatile() };
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        let paddr = unsafe {
            ACPI.get_pci_config_regions_addr(segment, bus, device, function)
                .unwrap()
        };
        let vaddr = phys_to_virt(PhysAddr::from(paddr as usize)).as_usize() as *mut u32;
        let address = unsafe { vaddr.add(offset as usize) };
        return unsafe { address.read_volatile() };
    }

    fn write_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u8) {
        let paddr = unsafe {
            ACPI.get_pci_config_regions_addr(segment, bus, device, function)
                .unwrap()
        };
        let vaddr = phys_to_virt(PhysAddr::from(paddr as usize)).as_usize() as *mut u8;
        let address = unsafe { vaddr.add(offset as usize) };
        return unsafe { address.write_volatile(value) };
    }

    fn write_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u16) {
        let paddr = unsafe {
            ACPI.get_pci_config_regions_addr(segment, bus, device, function)
                .unwrap()
        };
        let vaddr = phys_to_virt(PhysAddr::from(paddr as usize)).as_usize() as *mut u16;
        let address = unsafe { vaddr.add(offset as usize) };
        return unsafe { address.write_volatile(value) };
    }

    fn write_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u32) {
        let paddr = unsafe {
            ACPI.get_pci_config_regions_addr(segment, bus, device, function)
                .unwrap()
        };
        let vaddr = phys_to_virt(PhysAddr::from(paddr as usize)).as_usize() as *mut u32;
        let address = unsafe { vaddr.add(offset as usize) };
        return unsafe { address.write_volatile(value) };
    }
}

#[derive(Clone, Debug)]
struct LocalAllocator;

unsafe impl core::alloc::Allocator for LocalAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0)),
            size => {
                let raw_ptr = global_allocator()
                    .alloc(layout.size(), layout.align())
                    .unwrap() as *mut u8;
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(NonNull::slice_from_raw_parts(ptr, size))
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            global_allocator().dealloc(ptr.as_ptr() as usize, layout.size(), layout.align())
        }
    }
}

struct Acpi {
    rsdp: AcpiTables<LocalAcpiHandler>,
    aml_context: AmlContext,
}

/// irq model used in ACPI
pub enum X86IrqModel {
    /// PIC model
    PIC,
    /// APIC model
    APIC,
}

impl Acpi {
    pub unsafe fn new() -> Self {
        Acpi {
            rsdp: AcpiTables::search_for_rsdp_bios(LocalAcpiHandler).unwrap(),
            aml_context: AmlContext::new(Box::new(LocalAmlHandler), DebugVerbosity::None),
        }
    }

    fn init(&mut self) -> bool {
        let dsdt = self.rsdp.dsdt.as_ref().unwrap();
        let paddr = PhysAddr::from(dsdt.address);
        let vaddr = phys_to_virt(paddr).as_ptr();
        let slice = unsafe {
            core::slice::from_raw_parts_mut(vaddr as *mut u8, dsdt.length as usize)
        };
        if let Err(_) = self.aml_context.parse_table(slice) {
            return false;
        }
        self.set_irq_model(X86IrqModel::APIC)
    }

    /// Set IRQ model that ACPI uses by invoking ACPI global method _PIC.
    ///
    /// This method changes the routing tables (PIC or APIC) to return when calling _PRT methods.
    /// Since this method changes ACPI state, it could lead to concurrent problem.
    /// But currently it is only invoked in init thus runs by primary cpu only.
    /// We may need a lock for ACPI in the future as more ACPI state altering method implemented.
    fn set_irq_model(&mut self, irq_model: X86IrqModel) -> bool {
        let value = match irq_model {
            X86IrqModel::PIC => 0,
            X86IrqModel::APIC => 1,
        };
        let mut arg = aml::value::Args::EMPTY;
        if let Err(_) = arg.store_arg(0, aml::AmlValue::Integer(value)) {
            return false;
        }
        let result = self
            .aml_context
            .invoke_method(&AmlName::from_str("\\_PIC").unwrap(), arg);
        if let Err(err) = result {
            error!("set_irq_model failed:{:#?}", err);
            return false;
        }
        true
    }

    /// Get PCI IRQ by invoking device _PRT method.
    ///
    /// Each PCI bus that ACPI provides interrupt routing information for appears as a device
    /// in the ACPI namespace.
    /// Each of these devices contains a _PRT method that returns an array of objects describing
    /// the interrupt routing for slots on that PCI bus.
    fn get_pci_irq_desc(&mut self, bus: u8, device: u8, function: u8) -> Option<IrqDescriptor> {
        match AmlName::from_str(format!("\\_SB.PCI{bus_id}._PRT", bus_id = bus).as_str()) {
            Ok(prt_path) => {
                match PciRoutingTable::from_prt_path(&prt_path, &mut self.aml_context) {
                    Ok(table) => {
                        if let Ok(irq_descriptor) = table.route(
                            device as u16,
                            function as u16,
                            Pin::IntA,
                            &mut self.aml_context,
                        ) {
                            Some(irq_descriptor)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    /// Get base physical address of the PCIe ECAM space from ACPI MCFG table.
    ///
    /// Currently the ACPI crate does not export MCFG internal structure, thus we can not get ECAM
    /// space address directly. This method get configuration space address of bdf(0:0:0) instead.
    fn get_ecam_address(&mut self) -> Option<u64> {
        if let Ok(config) = acpi::mcfg::PciConfigRegions::new(&self.rsdp) {
            return Some(config.physical_address(0, 0, 0, 0).unwrap() as u64);
        }
        None
    }

    /// Get PCIe configuration space physical address of device function.
    fn get_pci_config_regions_addr(
        &mut self,
        segment_group_no: u16,
        bus: u8,
        device: u8,
        function: u8,
    ) -> Option<u64> {
        if let Ok(config) = acpi::mcfg::PciConfigRegions::new(&self.rsdp) {
            return config.physical_address(segment_group_no, bus, device, function);
        }
        None
    }
}

static mut ACPI: LazyInit<Acpi> = LazyInit::new();

pub(crate) fn init() {
    unsafe {
        let mut acpi = Acpi::new();
        acpi.init();
        ACPI.init_by(acpi);
    }
}

/// Get PCI IRQ and map it to vector used in OS.
pub fn get_pci_irq_vector(bus: u8, device: u8, function: u8) -> Option<usize> {
    if let Some(irq_desc) = unsafe { ACPI.get_pci_irq_desc(bus, device, function) } {
        Some(irq_to_vector(irq_desc.irq as u8))
    } else {
        None
    }
}

/// Get PCIe ECAM space physical address.
pub fn get_ecam_address() -> Option<u64> {
    unsafe { ACPI.get_ecam_address() }
}
