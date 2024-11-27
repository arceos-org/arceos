use core::ptr::NonNull;

use axhal::mem::phys_to_virt;
use nvme_driver::{Config, Namespace};
use pcie::{Chip, CommandRegister, DeviceType, Endpoint, RootComplex};

use crate::{BaseDriverOps, BlockDriverOps};

pub struct Nvme {
    inner: nvme_driver::Nvme,
    ns: Namespace,
}

unsafe impl Send for Nvme {}
unsafe impl Sync for Nvme {}

impl Nvme {
    pub fn new<C: Chip>(root: &mut RootComplex<C>, ep: &Endpoint) -> Option<Self> {
        ep.update_command(root, |cmd| {
            cmd | CommandRegister::IO_ENABLE
                | CommandRegister::MEMORY_ENABLE
                | CommandRegister::BUS_MASTER_ENABLE
        });

        if ep.device_type() == DeviceType::NvmeController {
            let bar_addr = match &ep.bar {
                pcie::BarVec::Memory32(bar_vec_t) => {
                    let bar0 = bar_vec_t[0].as_ref().unwrap();
                    bar0.address as usize
                }
                pcie::BarVec::Memory64(bar_vec_t) => {
                    let bar0 = bar_vec_t[0].as_ref().unwrap();
                    bar0.address as usize
                }
                pcie::BarVec::Io(_bar_vec_t) => return None,
            };

            let addr = phys_to_virt(bar_addr.into());

            let mut nvme = nvme_driver::Nvme::new(
                unsafe { NonNull::new_unchecked(addr.as_mut_ptr()) },
                Config {
                    page_size: 0x1000,
                    io_queue_pair_count: 1,
                },
            )
            .inspect_err(|e| error!("{:?}", e))
            .unwrap();
            let ns_list = nvme.namespace_list().ok()?;
            let ns = ns_list.first()?;

            return Some(Self {
                inner: nvme,
                ns: *ns,
            });
        }

        None
    }
}

impl BaseDriverOps for Nvme {
    fn device_name(&self) -> &str {
        "NVME"
    }

    fn device_type(&self) -> crate::DeviceType {
        crate::DeviceType::Block
    }
}

impl BlockDriverOps for Nvme {
    fn num_blocks(&self) -> u64 {
        self.ns.lba_count as _
    }

    fn block_size(&self) -> usize {
        self.ns.lba_size
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> crate::DevResult {
        self.inner
            .block_read_sync(&self.ns, block_id as _, buf)
            .map_err(|_e| crate::DevError::Io)
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> crate::DevResult {
        self.inner
            .block_write_sync(&self.ns, block_id, buf)
            .map_err(|_e| crate::DevError::Io)
    }

    fn flush(&mut self) -> crate::DevResult {
        Ok(())
    }
}
