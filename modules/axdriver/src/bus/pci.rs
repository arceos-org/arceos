use crate::{prelude::*, AllDevices};
use axhal::mem::phys_to_virt;
use driver_pci::{Cam, Command, DeviceFunction, HeaderType, PciRoot};

fn config_pci_device(root: &mut PciRoot, bdf: DeviceFunction) -> DevResult {
    // Enable the device.
    let (_status, cmd) = root.get_status_command(bdf);
    root.set_command(
        bdf,
        cmd | Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
    );
    Ok(())
}

impl AllDevices {
    pub(crate) fn probe_bus_devices(&mut self) {
        let base_vaddr = phys_to_virt(axconfig::PCI_ECAM_BASE.into());
        let mut root = unsafe { PciRoot::new(base_vaddr.as_mut_ptr(), Cam::Ecam) };

        for bus in 0..=axconfig::PCI_BUS_END as u8 {
            for (bdf, dev_info) in root.enumerate_bus(bus) {
                debug!("PCI {}: {}", bdf, dev_info);
                if dev_info.header_type != HeaderType::Standard {
                    continue;
                }
                match config_pci_device(&mut root, bdf) {
                    Ok(_) => for_each_drivers!(type Driver, {
                        if let Some(dev) = Driver::probe_pci(&mut root, bdf, &dev_info) {
                            info!(
                                "registered a new {:?} device at {}: {:?}",
                                dev.device_type(),
                                bdf,
                                dev.device_name(),
                            );
                            self.add_device(dev);
                            continue; // skip to the next device
                        }
                    }),
                    Err(e) => warn!(
                        "failed to enable PCI device at {}({}): {:?}",
                        bdf, dev_info, e
                    ),
                }
            }
        }
    }
}
