//! Address space of user mode management of microkernel version of ArceOS.
#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate axlog;

use core::cmp::Ordering;

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{collections::BTreeMap, string::String};
use axalloc::GlobalPage;
use axerrno::{ax_err, AxError, AxResult};
use axhal::{
    mem::{phys_to_virt, virt_to_phys},
    paging::{MappingFlags, PageTable},
};
use crate_interface::def_interface;
use memory_addr::{align_up, align_up_4k, PhysAddr, VirtAddr, PAGE_SIZE_4K};
use spinlock::SpinNoIrq;

/// Start of binary
pub const USER_START: usize = 0x0400_0000;
/// top of user stack
pub const USTACK_START: usize = 0x10_0000_0000 - USTACK_SIZE;
/// size of user stack
pub const USTACK_SIZE: usize = 4096 * 10;
/// address of trampoline page
pub const TRAMPOLINE_START: usize = 0xffff_ffc0_0000_0000;
/// start of default mmap areas
pub const MMAP_AREA_START: usize = 0x10_0000_0000;
/// end of default mmap areas
pub const MMAP_AREA_END: usize = 0x20_0000_0000;

pub(crate) struct MapSegment {
    start_vaddr: VirtAddr,
    size: usize,
    phy_mem: Vec<Arc<GlobalPage>>,
}

#[derive(Clone)]
pub(crate) struct HeapSegment {
    start_vaddr: VirtAddr,
    actual_size: usize,
}

/// mutable inner objects of address space
pub struct AddrSpaceInner {
    segments: alloc::vec::Vec<MapSegment>,
    page_table: PageTable,
    heap: Option<HeapSegment>,
    mmap_use: BTreeMap<VirtAddr, GlobalPage>,
}

impl Default for AddrSpaceInner {
    fn default() -> Self {
        Self::new()
    }
}

impl AddrSpaceInner {
    pub(crate) fn new() -> AddrSpaceInner {
        AddrSpaceInner {
            segments: vec![],
            page_table: PageTable::try_new().expect("Creating page table failed!"),
            heap: None,
            mmap_use: BTreeMap::new(),
        }
    }

    /// create a segment region
    pub fn add_region(
        &mut self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        phy_page: Arc<GlobalPage>,
        flags: MappingFlags,
        huge_page: bool,
    ) -> AxResult<()> {
        self.page_table
            .map_region(vaddr, paddr, phy_page.size(), flags, huge_page)
            .map_err(|_| AxError::BadAddress)?;
        self.segments.push(MapSegment {
            start_vaddr: vaddr,
            size: phy_page.size(),
            phy_mem: vec![phy_page],
        });
        Ok(())
    }

    /// create a segment region without memory allocation
    pub fn add_region_shadow(
        &mut self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        size: usize,
        flags: MappingFlags,
        huge_page: bool,
    ) -> AxResult<()> {
        self.page_table
            .map_region(vaddr, paddr, size, flags, huge_page)
            .map_err(|_| AxError::BadAddress)?;
        self.segments.push(MapSegment {
            start_vaddr: vaddr,
            size,
            phy_mem: vec![],
        });
        Ok(())
    }

    /// remove a segment
    pub fn remove_region(&mut self, vaddr: VirtAddr) -> AxResult<()> {
        if let Some((idx, _)) = self
            .segments
            .iter()
            .enumerate()
            .find(|(_, page)| page.start_vaddr == vaddr)
        {
            let item = self.segments.remove(idx);
            self.page_table
                .unmap_region(item.start_vaddr, item.size)
                .map_err(|_| AxError::BadAddress)?;
            Ok(())
        } else {
            ax_err!(BadAddress)
        }
    }

    /// get root address of pagetable
    pub fn page_table_addr(&self) -> PhysAddr {
        self.page_table.root_paddr()
    }

    pub(crate) fn init_heap(&mut self, vaddr: VirtAddr) {
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

    /// modify heap size
    pub fn sbrk(&mut self, size: isize) -> Option<usize> {
        if let Some(heap) = &mut self.heap {
            let old_brk: usize = (heap.start_vaddr + heap.actual_size).into();
            info!("user sbrk: {} bytes", size);
            match size.cmp(&0) {
                Ordering::Equal => {
                    return Some(old_brk);
                }
                Ordering::Less => {
                    if (-size) as usize > heap.actual_size {
                        return None;
                    }
                    heap.actual_size -= -size as usize
                }
                Ordering::Greater => {
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
                                        MappingFlags::READ
                                            | MappingFlags::WRITE
                                            | MappingFlags::USER,
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
            }
            Some(old_brk)
        } else {
            None
        }
    }

    /// a simple mmap-like page allocator, except that memory is alloced in pages
    /// @param addr: desired memory position
    /// @param len: desired pages
    /// @returns: starting addr of the maped pages
    pub fn mmap_page(
        &mut self,
        _addr: Option<VirtAddr>,
        len: usize,
        flags: MappingFlags,
    ) -> AxResult<VirtAddr> {
        let mut addr: VirtAddr = MMAP_AREA_START.into();
        let len = align_up_4k(len);
        let pages = len / PAGE_SIZE_4K;
        while addr + len < MMAP_AREA_END.into() {
            if let Some(offset) = (0..pages)
                .find(|offset| self.mmap_use.contains_key(&(addr + offset * PAGE_SIZE_4K)))
            {
                addr += (offset + 1) * PAGE_SIZE_4K;
            } else {
                // TODO: undo when error
                (0..pages).for_each(|offset| {
                    let phy_page = GlobalPage::alloc_zero().expect("Run out of memory!");
                    self.page_table
                        .map_region(
                            addr + offset * PAGE_SIZE_4K,
                            phy_page.start_paddr(virt_to_phys),
                            PAGE_SIZE_4K,
                            flags,
                            false,
                        )
                        .expect("Mapping error");
                    self.mmap_use.insert(addr + offset * PAGE_SIZE_4K, phy_page);
                });
                return Ok(addr);
            }
        }
        ax_err!(NoMemory)
    }

    /// unmap a page obtained from `mmap`
    pub fn munmap_page(&mut self, addr: VirtAddr, len: usize) -> AxResult<()> {
        let len = align_up_4k(len);
        trace!("unmap: [{:x?}, {:x?})", addr, addr + len);
        let pages = len / PAGE_SIZE_4K;
        if (0..pages).any(|offset| !self.mmap_use.contains_key(&(addr + offset * PAGE_SIZE_4K))) {
            return ax_err!(BadAddress);
        } else {
            (0..pages).for_each(|offset| {
                self.page_table.unmap(addr + offset * PAGE_SIZE_4K).unwrap();
                self.mmap_use.remove(&(addr + offset * PAGE_SIZE_4K));
            })
        }
        Ok(())
    }

    /// translate a slice to slices in kernel
    pub fn translate_buffer(
        &self,
        vaddr: VirtAddr,
        size: usize,
        _write: bool,
    ) -> AxResult<Vec<&'static mut [u8]>> {
        let mut read_size = 0usize;
        let mut vaddr = vaddr;
        let mut result: Vec<&'static mut [u8]> = vec![];
        while read_size < size {
            let (paddr, _flag, page_size) = self
                .page_table
                .query(vaddr)
                .map_err(|_| AxError::BadAddress)?;
            /*
                if !flag.contains(MappingFlags::USER) || (write && !flag.contains(MappingFlags::WRITE)) {
                panic!("Invalid vaddr with improper rights!");
            }
                 */
            let nxt_vaddr = align_up(vaddr.as_usize() + 1, page_size.into());
            let len = (nxt_vaddr - vaddr.as_usize()).min(size - read_size);
            let data =
                unsafe { core::slice::from_raw_parts_mut(phys_to_virt(paddr).as_mut_ptr(), len) };
            debug!("translating {:x} -> {:x}, len = {}", vaddr, paddr, len);
            vaddr += len;
            read_size += len;
            result.push(data);
        }
        Ok(result)
    }

    /// find a mapping of the virtual address
    pub fn query(&self, vaddr: VirtAddr) -> Option<PhysAddr> {
        self.page_table.query(vaddr).ok().map(|x| x.0)
    }
}

impl AddrSpaceInner {
    fn init_global(&mut self, data: &[u8]) -> AxResult<()> {
        let segments = elf_loader::SegmentEntry::new(data).ok_or(AxError::InvalidData)?;

        let mut data_end: VirtAddr = 0.into();

        for segment in &segments {
            let mut user_phy_page = GlobalPage::alloc_contiguous(
                align_up_4k(segment.size) / PAGE_SIZE_4K,
                PAGE_SIZE_4K,
            )
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

            self.add_region(
                segment.start_addr,
                user_phy_page.start_paddr(virt_to_phys),
                Arc::new(user_phy_page),
                segment.flags | MappingFlags::USER,
                false,
            )
            .map_err(|_| AxError::BadAddress)?;
            data_end = data_end.max(segment.start_addr + align_up_4k(segment.size))
        }

        self.init_heap(data_end);

        /*
        // stack allocation
        //assert!(USTACK_SIZE % PAGE_SIZE_4K == 0);
        #[cfg(not(feature = "multitask"))]
        {
            let user_stack_page =
                GlobalPage::alloc_contiguous(USTACK_SIZE / PAGE_SIZE_4K, PAGE_SIZE_4K)
                    .expect("Alloc page error!");
            debug!("{:?}", user_stack_page);

            self
                .add_region(
                    USTACK_START.into(),
                    user_stack_page.start_paddr(virt_to_phys),
                    Arc::new(user_stack_page),
                    MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
                    false,
                )
                .expect("Memory Error");
        }
        */
        extern "C" {
            fn strampoline();
        }
        self.add_region_shadow(
            TRAMPOLINE_START.into(),
            virt_to_phys((strampoline as usize).into()),
            PAGE_SIZE_4K,
            MappingFlags::READ | MappingFlags::EXECUTE,
            false,
        )
        .expect("Memory Error");

        Ok(())
    }
}

impl Clone for AddrSpace {
    fn clone(&self) -> Self {
        let inner = self.lock();
        let mut new_inner = AddrSpaceInner::new();
        let page_table = &mut new_inner.page_table;

        for segment in &inner.segments {
            let mut new_seg = MapSegment {
                start_vaddr: segment.start_vaddr,
                size: segment.size,
                phy_mem: Vec::new(),
            };
            let pages = segment.size / PAGE_SIZE_4K;

            /*
                if segment.phy_mem.is_empty() { // pages such as trampoline
                (0..pages).for_each(|page| {
                let vaddr = segment.start_vaddr + page * PAGE_SIZE_4K;
                let (paddr, flags, _) = inner.page_table.query(vaddr).unwrap();
                page_table.map_region(vaddr, paddr, PAGE_SIZE_4K, flags, false).unwrap();
            })
            } else {
                 */
            (0..pages).for_each(|page| {
                let vaddr = segment.start_vaddr + page * PAGE_SIZE_4K;
                let (paddr, flags, _) = inner.page_table.query(vaddr).unwrap();
                let mut user_phy_page = GlobalPage::alloc().unwrap();
                user_phy_page.as_slice_mut().copy_from_slice(unsafe {
                    core::slice::from_raw_parts(phys_to_virt(paddr).as_ptr(), PAGE_SIZE_4K)
                });
                page_table
                    .map_region(
                        vaddr,
                        user_phy_page.start_paddr(virt_to_phys),
                        PAGE_SIZE_4K,
                        flags,
                        false,
                    )
                    .unwrap();
                new_seg.phy_mem.push(user_phy_page.into());
            });
            //}
            new_inner.segments.push(new_seg)
        }
        new_inner.heap = inner.heap.clone();
        for (vaddr, page) in &inner.mmap_use {
            let mut user_phy_page = GlobalPage::alloc().unwrap();
            user_phy_page
                .as_slice_mut()
                .copy_from_slice(page.as_slice());
            let (_, flags, _) = inner.page_table.query(*vaddr).unwrap();

            page_table
                .map_region(
                    *vaddr,
                    user_phy_page.start_paddr(virt_to_phys),
                    PAGE_SIZE_4K,
                    flags,
                    false,
                )
                .unwrap();
            new_inner.mmap_use.insert(*vaddr, user_phy_page);
        }

        AddrSpace(SpinNoIrq::new(new_inner))
    }
}

/// A user address space
pub struct AddrSpace(SpinNoIrq<AddrSpaceInner>);

impl AddrSpace {
    /// init address space of a process
    pub fn init_global(user_elf: &[u8]) -> AxResult<AddrSpace> {
        let mut user_space = AddrSpaceInner::new();
        user_space.init_global(user_elf)?;

        Ok(AddrSpace(SpinNoIrq::new(user_space)))
    }

    /// init address space in current structure.
    pub fn init_exec(&self, user_elf: &[u8]) -> AxResult<()> {
        let mut new_space = AddrSpaceInner::new();
        new_space.init_global(user_elf)?;
        *self.0.lock() = new_space;
        Ok(())
    }
}

impl core::ops::Deref for AddrSpace {
    type Target = SpinNoIrq<AddrSpaceInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// interface to get current process's address space
#[def_interface]
pub trait CurrentAddrSpace {
    /// get reference of current address space
    fn current_addr_space() -> Arc<AddrSpace>;
}

fn current_addr_space() -> Arc<AddrSpace> {
    crate_interface::call_interface!(CurrentAddrSpace::current_addr_space)
}

/// alloc a user space
pub fn alloc_user_page(vaddr: VirtAddr, size: usize, flags: MappingFlags) -> Arc<GlobalPage> {
    let mut user_phy_page =
        GlobalPage::alloc_contiguous(align_up_4k(size) / PAGE_SIZE_4K, PAGE_SIZE_4K)
            .expect("Alloc page error!");
    // init
    user_phy_page.zero();
    let user_phy_page = Arc::new(user_phy_page);

    current_addr_space()
        .lock()
        .add_region(
            vaddr,
            user_phy_page.start_paddr(virt_to_phys),
            user_phy_page.clone(),
            flags,
            false,
        )
        .expect("Memory Error");

    user_phy_page
}

/// manage heap space of current process
pub fn global_sbrk(size: isize) -> Option<usize> {
    current_addr_space().lock().sbrk(size)
}

/// get root of page table of current process
pub fn get_satp() -> usize {
    current_addr_space().lock().page_table_addr().into()
}

/// mmap a page of current process
pub fn mmap_page(addr: Option<VirtAddr>, len: usize, flags: MappingFlags) -> AxResult<VirtAddr> {
    current_addr_space().lock().mmap_page(addr, len, flags)
}

/// unmap a page of current process
pub fn munmap_page(addr: VirtAddr, len: usize) -> AxResult<()> {
    current_addr_space().lock().munmap_page(addr, len)
}

/// translate a slice from current process
pub fn translate_buffer(vaddr: VirtAddr, size: usize, write: bool) -> Vec<&'static mut [u8]> {
    current_addr_space()
        .lock()
        .translate_buffer(vaddr, size, write)
        .unwrap()
}

/// copy a slice of current process into kernel address space
pub fn copy_slice_from_user(vaddr: VirtAddr, size: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let buffers = translate_buffer(vaddr, size, false);
    for fragment in &buffers {
        result.extend_from_slice(fragment);
    }
    result
}

/// copy a `str` in current process into kernel address space
pub fn copy_str_from_user(vaddr: VirtAddr, size: usize) -> String {
    let result = copy_slice_from_user(vaddr, size);
    String::from_utf8(result).expect("Invalid string!")
}

/// translate a virtual address to phyical address of current process
pub fn translate_addr(vaddr: VirtAddr) -> Option<PhysAddr> {
    current_addr_space().lock().query(vaddr)
}

/// Copy a [u8] array `data' from current memory space into position `ptr' of the userspace `token'(not used)
// Copied from my code in rCore
pub fn copy_byte_buffer_to_user(_token: usize, ptr: *const u8, data: &[u8]) {
    let copy_len = data.len();
    let dst = translate_buffer((ptr as usize).into(), copy_len, false);
    let mut offset = 0;
    for dst_space in dst {
        let dst_len = dst_space.len();
        dst_space.copy_from_slice(&data[offset..offset + dst_len]);
        offset += dst_len;
    }
    assert_eq!(copy_len, offset);
}

/// Copy a `data' with type `T' from current memory space into position `ptr' of the userspace `token' (not used)
// Copied from my code in rCore
pub fn copy_data_to_user<T>(token: usize, ptr: *const u8, data: &T) {
    let data_ptr = data as *const T as *const u8;
    let data_buf = unsafe { core::slice::from_raw_parts(data_ptr, core::mem::size_of::<T>()) };
    copy_byte_buffer_to_user(token, ptr, data_buf);
}
