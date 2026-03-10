extern crate alloc;

use fdt_edit::PciSpace;
use rdrive::{
    PlatformDevice, module_driver,
    probe::{OnProbeError, fdt::NodeType, pci::*},
    register::FdtInfo,
};

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
    let NodeType::Pci(node) = info.node else {
        return Err(OnProbeError::NotMatch);
    };

    let regs = node.regs();
    for reg in &regs {
        trace!(
            "pcie reg: {:#x}, bus: {:#x}",
            reg.address, reg.child_bus_address
        );
    }

    let reg = regs
        .first()
        .ok_or_else(|| OnProbeError::other("PCIe controller has no regs"))?;
    let mmio_base = reg.address as usize;
    let mmio_size = reg.size.unwrap_or(0x1000) as usize;
    let mut drv = new_driver_generic(mmio_base, mmio_size, &crate::boot::Kernel).map_err(|e| {
        OnProbeError::other(alloc::format!("failed to create PCIe controller: {e:?}"))
    })?;

    for range in node.ranges().unwrap_or_default() {
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
