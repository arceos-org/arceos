extern crate alloc;

use alloc::format;
use arm_gic_driver::v3::Gic;
use rdrive::{PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo};

use crate::dyn_drivers::iomap;

module_driver!(
    name: "GICv3",
    level: ProbeLevel::PreKernel,
    priority: ProbePriority::INTC,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,gic-v3"],
            on_probe: probe_gic
        }
    ],
);

fn probe_gic(info: FdtInfo<'_>, dev: PlatformDevice) -> Result<(), OnProbeError> {
    let mut reg = info.node.reg().ok_or(OnProbeError::other(format!(
        "[{}] has no reg",
        info.node.name()
    )))?;

    let gicd_reg = reg.next().unwrap();
    let gicr_reg = reg.next().unwrap();

    let gicd = iomap(
        (gicd_reg.address as usize).into(),
        gicd_reg.size.unwrap_or(0x1000),
    )?;
    let gicr = iomap(
        (gicr_reg.address as usize).into(),
        gicr_reg.size.unwrap_or(0x1000),
    )?;

    let gic = unsafe { Gic::new(gicd.into(), gicr.into()) };

    dev.register(rdif_intc::Intc::new(gic));

    Ok(())
}
