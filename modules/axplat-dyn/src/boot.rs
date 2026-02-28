use core::ptr::NonNull;

use axerrno::AxError;
use axklib::AxResult;
use axplat::mem::phys_to_virt;
use somehal::{KernelOp, setup::*};

#[somehal::entry(Kernel)]
fn main() -> ! {
    let mut args = 0;
    if let Some(fdt) = somehal::fdt_addr_phys() {
        args = fdt;
    }

    axplat::call_main(0, args)
}

#[somehal::secondary_entry]
fn secondary_main() {
    axplat::call_secondary_main(meta.cpu_idx);
}

pub struct Kernel;

impl KernelOp for Kernel {}

impl MmioOp for Kernel {
    fn ioremap(&self, addr: MmioAddr, size: usize) -> Result<Mmio, Error> {
        let virt = match axklib::mem::iomap(addr.as_usize().into(), size) {
            Ok(v) => v,
            Err(AxError::AlreadyExists) => {
                // If the region is already mapped, just return the existing mapping.
                phys_to_virt(addr.as_usize().into())
            }
            Err(e) => return Err(anyhow::anyhow!("{e:?}")),
        };
        Ok(unsafe { Mmio::new(addr, NonNull::new(virt.as_mut_ptr()).unwrap(), size) })
    }

    fn iounmap(&self, _mmio: &Mmio) {}
}
