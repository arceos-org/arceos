extern crate alloc;

use alloc::format;
use arm_gic_driver::v2::{Gic, HyperAddress};
use rdrive::{PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo};

use crate::dyn_drivers::iomap;

module_driver!(
    name: "GICv2",
    level: ProbeLevel::PreKernel,
    priority: ProbePriority::INTC,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,cortex-a15-gic", "arm,gic-400"],
            on_probe: probe_gic
        },
    ] ,
);

fn probe_gic(info: FdtInfo<'_>, dev: PlatformDevice) -> Result<(), OnProbeError> {
    let mut reg = info.node.reg().ok_or(OnProbeError::other(format!(
        "[{}] has no reg",
        info.node.name()
    )))?;

    let gicd_reg = reg.next().unwrap();
    let gicc_reg = reg.next().unwrap();

    let gicd = iomap(
        (gicd_reg.address as usize).into(),
        gicd_reg.size.unwrap_or(0x1000),
    )?;
    let gicc = iomap(
        (gicc_reg.address as usize).into(),
        gicc_reg.size.unwrap_or(0x1000),
    )?;

    let mut hyper = None;

    if let Some(gich_reg) = reg.next() {
        if let Some(gicv_reg) = reg.next() {
            let gich = iomap(
                (gich_reg.address as usize).into(),
                gich_reg.size.unwrap_or(0x1000),
            )?;
            let gicv = iomap(
                (gicv_reg.address as usize).into(),
                gicv_reg.size.unwrap_or(0x1000),
            )?;

            hyper = Some(HyperAddress::new(gich.into(), gicv.into()))
        }
    }

    let gic = unsafe { Gic::new(gicd.into(), gicc.into(), hyper) };

    dev.register(rdif_intc::Intc::new(gic));

    Ok(())
}
