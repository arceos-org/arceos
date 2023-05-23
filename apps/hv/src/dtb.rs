use arrayvec::ArrayVec;
use fdt::Fdt;

#[derive(Clone, Debug)]
pub struct Device {
    pub base_address: usize,
    pub size: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MachineMeta {
    pub physical_memory_offset: usize,
    pub physical_memory_size: usize,

    pub virtio: ArrayVec<Device, 16>,

    pub test_finisher_address: Option<Device>,

    pub uart: Option<Device>,

    pub clint: Option<Device>,

    pub plic: Option<Device>,

    pub pci: Option<Device>,
}

impl MachineMeta {
    pub fn parse(dtb: usize) -> Self {
        let fdt = unsafe { Fdt::from_ptr(dtb as *const u8) }.unwrap();
        let memory = fdt.memory();
        let mut meta = MachineMeta::default();
        for region in memory.regions() {
            meta.physical_memory_offset = region.starting_address as usize;
            meta.physical_memory_size = region.size.unwrap();
        }
        // probe virtio mmio device
        for node in fdt.find_all_nodes("/soc/virtio_mmio") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                meta.virtio.push(Device {
                    base_address: paddr,
                    size,
                })
            }
        }
        meta.virtio.sort_unstable_by_key(|v| v.base_address);

        // probe virt test
        for node in fdt.find_all_nodes("/soc/test") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("test addr: {:#x}, size: {:#x}", base_addr, size);
                meta.test_finisher_address = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        // probe uart device
        for node in fdt.find_all_nodes("/soc/uart") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("UART addr: {:#x}, size: {:#x}", base_addr, size);
                meta.uart = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        // probe clint(core local interrupter)
        for node in fdt.find_all_nodes("/soc/clint") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("CLINT addr: {:#x}, size: {:#x}", base_addr, size);
                meta.clint = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        // probe plic
        for node in fdt.find_all_nodes("/soc/plic") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("PLIC addr: {:#x}, size: {:#x}", base_addr, size);
                meta.plic = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        for node in fdt.find_all_nodes("/soc/pci") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("PCI addr: {:#x}, size: {:#x}", base_addr, size);
                meta.pci = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        meta
    }
}
