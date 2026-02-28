use axplat::mem::{MemIf, PhysAddr, RawRange, VirtAddr};
use heapless::Vec;
use somehal::mem::MemoryType;
use spin::Once;

static FREE_LIST: Once<Vec<RawRange, 32>> = Once::new();
static RESERVED_LIST: Once<Vec<RawRange, 32>> = Once::new();
static MMIO_LIST: Once<Vec<RawRange, 16>> = Once::new();

struct MemIfImpl;

#[impl_plat_interface]
impl MemIf for MemIfImpl {
    fn phys_ram_ranges() -> &'static [RawRange] {
        FREE_LIST.call_once(|| {
            let mut list = Vec::new();
            for r in somehal::mem::memory_map() {
                if matches!(r.memory_type, MemoryType::Free) {
                    list.push((r.physical_start, r.size_in_bytes)).unwrap();
                }
            }
            list
        })
    }

    fn reserved_phys_ram_ranges() -> &'static [RawRange] {
        RESERVED_LIST.call_once(|| {
            let mut list = Vec::new();
            for r in somehal::mem::memory_map() {
                if matches!(
                    r.memory_type,
                    MemoryType::Reserved | MemoryType::KImage | MemoryType::PerCpuData
                ) {
                    list.push((r.physical_start, r.size_in_bytes)).unwrap();
                }
            }
            list
        })
    }

    fn mmio_ranges() -> &'static [RawRange] {
        MMIO_LIST.call_once(|| {
            let mut list = Vec::new();
            for r in somehal::mem::memory_map() {
                if matches!(r.memory_type, MemoryType::Mmio) {
                    list.push((r.physical_start, r.size_in_bytes)).unwrap();
                }
            }
            list
        })
    }

    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        (somehal::mem::phys_to_virt(paddr.as_usize()) as usize).into()
    }

    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        somehal::mem::virt_to_phys(vaddr.as_ptr()).into()
    }

    fn kernel_aspace() -> (VirtAddr, usize) {
        let range = somehal::mem::kernel_space();
        (range.start.into(), range.len())
    }
}

#[unsafe(no_mangle)]
fn _percpu_base_ptr(idx: usize) -> *mut u8 {
    somehal::smp::percpu_data_ptr(idx).unwrap_or_default()
}
