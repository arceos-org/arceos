#[allow(unused_imports)]
use crate::{prelude::*, AllDevices};
#[cfg(target_arch = "aarch64")]
use arm_gic::gic_irq_translate;

impl AllDevices {
    pub(crate) fn probe_bus_devices(&mut self) {
        #[cfg(feature = "virtio")]
        dtb::compatible_nodes("virtio,mmio", |n| {
            let reg = n.prop("reg");
            //todo: check size of data
            let reg_base = reg.u64(0) as usize;
            let reg_size = reg.u64(1) as usize;
            //todo: get irq_num for riscv
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            let irq_num = None;
            #[cfg(target_arch = "aarch64")]
            let irq_num = n
                .find_prop("interrupts")
                .map(|irq| gic_irq_translate(irq.u32(0), irq.u32(1)) as usize);
            #[cfg(target_arch = "x86_64")]
            let irq_num = None;
            for_each_drivers!(type Driver, {
                if let Some(dev) = Driver::probe_mmio(
                    reg_base,
                    reg_size,
                    irq_num
                ) {
                    info!(
                        "registered a new {:?} device at [PA:{:#x}, PA:{:#x}): {:?}",
                        dev.device_type(),
                        reg_base, reg_base + reg_size,
                        dev.device_name(),
                    );
                    self.add_device(dev);
                    return;
                }
            });
        });
    }
}
