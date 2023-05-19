#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate axlog;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use axalloc::GlobalPage;
use axhal::{
    mem::{phys_to_virt, virt_to_phys},
    paging::{MappingFlags, PageTable},
};
use lazy_init::LazyInit;
use memory_addr::{align_up, align_up_4k, PhysAddr, VirtAddr, PAGE_SIZE_4K};
use spinlock::SpinNoIrq;

pub const USER_START: usize = 0x0400_0000;
pub const USTACK_START: usize = 0xf_ffff_f000;
pub const USTACK_SIZE: usize = 4096;
pub const TRAMPOLINE_START: usize = 0xffff_ffc0_0000_0000;

pub struct MapSegment {
    start_vaddr: VirtAddr,
    size: usize,
    phy_mem: Vec<Arc<GlobalPage>>,
}

pub struct HeapSegment {
    start_vaddr: VirtAddr,
    actual_size: usize,
}

pub struct AddrSpace {
    segments: alloc::vec::Vec<MapSegment>,
    page_table: PageTable,
    heap: Option<HeapSegment>,
}

impl AddrSpace {
    pub fn new() -> AddrSpace {
        AddrSpace {
            segments: vec![],
            page_table: PageTable::try_new().expect("Creating page table failed!"),
            heap: None,
        }
    }

    pub fn add_region(
        &mut self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        phy_page: Arc<GlobalPage>,
        flags: MappingFlags,
        huge_page: bool,
    ) {
        self.page_table
            .map_region(vaddr, paddr, phy_page.size(), flags, huge_page)
            .expect("Mapping Segment Error");
        self.segments.push(MapSegment {
            start_vaddr: vaddr,
            size: phy_page.size(),
            phy_mem: vec![phy_page],
        })
    }
    pub fn add_region_shadow(
        &mut self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        size: usize,
        flags: MappingFlags,
        huge_page: bool,
    ) {
        self.page_table
            .map_region(vaddr, paddr, size, flags, huge_page)
            .expect("Mapping Segment Error");
        self.segments.push(MapSegment {
            start_vaddr: vaddr,
            size,
            phy_mem: vec![],
        })
    }

    pub fn page_table_addr(&self) -> PhysAddr {
        self.page_table.root_paddr()
    }

    pub fn init_heap(&mut self, vaddr: VirtAddr) {
        if self.heap.is_some() {
            return;
        }
        self.heap = Some(HeapSegment {
            start_vaddr: vaddr,
            actual_size: 0,
        });
        let page = GlobalPage::alloc_zero().expect("Alloc error!");
        self.page_table
            .map_region(
                vaddr,
                page.start_paddr(virt_to_phys),
                page.size(),
                MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
                false,
            )
            .expect("Mapping Segment Error");
        self.segments.push(MapSegment {
            start_vaddr: vaddr,
            size: page.size(),
            phy_mem: vec![page.into()],
        });
        info!("User heap inited @ {:x}", vaddr);
    }

    pub fn sbrk(&mut self, size: isize) -> Option<usize> {
        if let Some(heap) = &mut self.heap {
            let old_brk: usize = (heap.start_vaddr + heap.actual_size).into();
            info!("user sbrk: {} bytes", size);
            if size == 0 {
                return Some(old_brk);
            } else if size < 0 {
                if (-size) as usize > heap.actual_size {
                    return None;
                }
                heap.actual_size -= -size as usize
            } else {
                heap.actual_size += size as usize;
                let heap_seg = self
                    .segments
                    .iter_mut()
                    .find(|x| x.start_vaddr == heap.start_vaddr)
                    .unwrap();
                if heap.actual_size > heap_seg.size {
                    let delta = align_up_4k(heap.actual_size - heap_seg.size);
                    while heap.actual_size > heap_seg.size {
                        if let Ok(page) =
                            GlobalPage::alloc_contiguous(delta / PAGE_SIZE_4K, PAGE_SIZE_4K)
                        {
                            self.page_table
                                .map_region(
                                    heap.start_vaddr + heap_seg.size,
                                    page.start_paddr(virt_to_phys),
                                    page.size(),
                                    MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
                                    false,
                                )
                                .expect("Mapping Error");
                            heap_seg.size += page.size();
                            heap_seg.phy_mem.push(page.into());
                        } else {
                            return None;
                        }
                    }
                }
            }
            Some(old_brk)
        } else {
            None
        }
    }
}

static mut GLOBAL_USER_ADDR_SPACE: LazyInit<SpinNoIrq<AddrSpace>> = LazyInit::new();

pub fn init_global_addr_space() {
    use axhal::mem::PAGE_SIZE_4K;
    extern crate alloc;

    extern "C" {
        fn ustart();
        fn uend();
    }

    let user_elf: &[u8] = unsafe {
        let len = (uend as usize) - (ustart as usize);
        core::slice::from_raw_parts(ustart as *const _, len)
    };

    debug!("{:x} {:x}", ustart as usize, user_elf.len());

    let segments = elf_loader::SegmentEntry::new(user_elf).expect("Corrupted elf file!");

    let mut user_space = AddrSpace::new();

    let mut data_end: VirtAddr = 0.into();

    for segment in &segments {
        let mut user_phy_page =
            GlobalPage::alloc_contiguous(align_up_4k(segment.size) / PAGE_SIZE_4K, PAGE_SIZE_4K)
                .expect("Alloc page error!");
        // init
        user_phy_page.zero();

        // copy user content
        user_phy_page.as_slice_mut()[..segment.data.len()].copy_from_slice(segment.data);
        debug!(
            "{:x} {:x}",
            user_phy_page.as_slice()[0],
            user_phy_page.as_slice()[1]
        );

        user_space.add_region(
            segment.start_addr,
            user_phy_page.start_paddr(virt_to_phys),
            Arc::new(user_phy_page),
            segment.flags | MappingFlags::USER,
            false,
        );
        data_end = data_end.max(segment.start_addr + align_up_4k(segment.size))
    }

    user_space.init_heap(data_end);

    // stack allocation
    assert!(USTACK_SIZE % PAGE_SIZE_4K == 0);
    #[cfg(not(feature = "multitask"))]
    {
        let user_stack_page =
            GlobalPage::alloc_contiguous(USTACK_SIZE / PAGE_SIZE_4K, PAGE_SIZE_4K)
                .expect("Alloc page error!");
        debug!("{:?}", user_stack_page);

        user_space.add_region(
            USTACK_START.into(),
            user_stack_page.start_paddr(virt_to_phys),
            Arc::new(user_stack_page),
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
            false,
        );
    }

    extern "C" {
        fn strampoline();
    }
    user_space.add_region_shadow(
        TRAMPOLINE_START.into(),
        virt_to_phys((strampoline as usize).into()),
        PAGE_SIZE_4K,
        MappingFlags::READ | MappingFlags::EXECUTE,
        false,
    );
    unsafe {
        GLOBAL_USER_ADDR_SPACE.init_by(SpinNoIrq::new(user_space));
    }
}

pub fn alloc_user_page(vaddr: VirtAddr, size: usize, flags: MappingFlags) -> Arc<GlobalPage> {
    let mut user_phy_page =
        GlobalPage::alloc_contiguous(align_up_4k(size) / PAGE_SIZE_4K, PAGE_SIZE_4K)
            .expect("Alloc page error!");
    // init
    user_phy_page.zero();
    let user_phy_page = Arc::new(user_phy_page);

    unsafe {
        GLOBAL_USER_ADDR_SPACE.lock().add_region(
            vaddr,
            user_phy_page.start_paddr(virt_to_phys),
            user_phy_page.clone(),
            flags,
            false,
        );
    }

    user_phy_page
}

pub fn global_sbrk(size: isize) -> Option<usize> {
    unsafe {
        GLOBAL_USER_ADDR_SPACE.lock().sbrk(size)
    }
}

pub fn get_satp() -> usize {
    unsafe {
        GLOBAL_USER_ADDR_SPACE
            .lock()
            .page_table_addr()
            .into()
    }
}

pub fn translate_buffer(vaddr: VirtAddr, size: usize, write: bool) -> Vec<&'static mut [u8]> {
    let addr_space = unsafe {GLOBAL_USER_ADDR_SPACE.lock()};

    let mut read_size = 0usize;
    let mut vaddr = vaddr;
    let mut result: Vec<&'static mut [u8]> = vec![];
    while read_size < size {
        let (paddr, flag, page_size) = addr_space.page_table.query(vaddr).expect("Invalid vaddr!");
        if !flag.contains(MappingFlags::USER) || (write && !flag.contains(MappingFlags::WRITE)) {
            panic!("Invalid vaddr with improper rights!");
        }

        let nxt_vaddr = align_up(vaddr.as_usize() + 1, page_size.into());
        let len = (nxt_vaddr - vaddr.as_usize()).min(size - read_size);
        let data =
            unsafe { core::slice::from_raw_parts_mut(phys_to_virt(paddr).as_mut_ptr(), len) };
        debug!("translating {:x} -> {:x}, len = {}", vaddr, paddr, len);
        vaddr += len;
        read_size += len;
        result.push(data);
    }
    result
}

pub fn copy_slice(vaddr: VirtAddr, size: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let buffers = translate_buffer(vaddr, size, false);
    for fragment in &buffers {
        result.extend_from_slice(fragment);
    }
    result
}
pub fn copy_str(vaddr: VirtAddr, size: usize) -> String {
    let result = copy_slice(vaddr, size);
    String::from_utf8(result).expect("Invalid string!")    
}

pub fn translate_addr(vaddr: VirtAddr) -> Option<PhysAddr> {
    unsafe {
        GLOBAL_USER_ADDR_SPACE.lock().page_table.query(vaddr).ok().map(|x| x.0)
    }
}

/// Copy a [u8] array `data' from current memory space into position `ptr' of the userspace `token'
// Copied from my code in rCore
pub fn copy_byte_buffer(_token: usize, ptr: *const u8, data: &[u8]) {
    let copy_len = data.len();
    let dst = translate_buffer((ptr as usize).into(), copy_len, true);
    let mut offset = 0;
    for dst_space in dst {
        let dst_len = dst_space.len();
        dst_space.copy_from_slice(&data[offset..offset + dst_len]);
        offset += dst_len;
    }
    assert_eq!(copy_len, offset);
}

/// Copy a `data' with type `T' from current memory space into position `ptr' of the userspace `token'
// Copied from my code in rCore
pub fn copy_data<T>(token: usize, ptr: *const u8, data: &T) {
    let data_ptr = data as *const T as *const u8;
    let data_buf = unsafe { core::slice::from_raw_parts(data_ptr, core::mem::size_of::<T>()) };
    copy_byte_buffer(token, ptr, data_buf);
}
