extern crate alloc;

use crate::dyn_drivers::iomap;
use memory_addr::MemoryAddr;
use rdrive::probe::fdt::PciSpace;
use rdrive::probe::pci::*;
use rdrive::{PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo};

module_driver!(
    name: "Generic PCIe Controller Driver",
    level: ProbeLevel::PostKernel,
    priority: ProbePriority::DEFAULT,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["pci-host-ecam-generic"],
            on_probe: probe
        }
    ],
);

fn probe(info: FdtInfo<'_>, plat_dev: PlatformDevice) -> Result<(), OnProbeError> {
    let node = info.node.into_pci().unwrap();
    let mut pcie_regs = alloc::vec![];
    for reg in node.node.reg().unwrap() {
        trace!(
            "pcie reg: {:#x}, bus: {:#x}",
            reg.address, reg.child_bus_address
        );
        let end = (reg.address as usize + reg.size.unwrap_or_default()).align_up_4k();
        let start = (reg.address as usize).align_down_4k();
        let size = end - start;
        pcie_regs.push(iomap(start.into(), size)?);
    }

    let base_vaddr = pcie_regs[0];
    let mut drv = new_driver_generic(base_vaddr);

    for range in node.ranges().unwrap() {
        debug!("pcie range {range:?}");
        match range.space {
            PciSpace::Memory32 => {
                drv.set_mem32(
                    PciMem32 {
                        address: range.cpu_address as _,
                        size: range.size as _,
                    },
                    range.prefetchable,
                );
            }
            PciSpace::Memory64 => {
                drv.set_mem64(
                    PciMem64 {
                        address: range.cpu_address as _,
                        size: range.size as _,
                    },
                    range.prefetchable,
                );
            }
            _ => {}
        }
    }

    plat_dev.register_pcie(drv);

    Ok(())
}
