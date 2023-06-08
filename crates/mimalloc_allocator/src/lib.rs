//! Mimalloc(single thread) for `no_std` systems.
//! written by rust code

#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::alloc::{AllocError, Layout};
use core::cmp::max;
use core::mem::size_of;

mod data;
use data::*;

/// the heap structure of the allocator
pub struct Heap {
    // 指向heap的地址
    pub addr: usize,
    // 尚未建成段的起始地址
    pub unused_begin: usize,
    // 尚未建成段的终止地址
    pub unused_end: usize,
    // 一个临时的尚未建成段的起始地址，为建立huge segment而暂存
    pub unused_begin_tmp: usize,
    // 一个临时的尚未建成段的终止地址，为建立huge segment而暂存
    pub unused_end_tmp: usize,
}

unsafe impl Send for Heap {}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            addr: 0,
            unused_begin: 0,
            unused_end: 0,
            unused_begin_tmp: 0,
            unused_end_tmp: 0,
        }
    }

    /// get reference
    pub fn get_ref(&self) -> &MiHeap {
        unsafe { &(*(self.addr as *const MiHeap)) }
    }
    /// get mut reference
    pub fn get_mut_ref(&mut self) -> &mut MiHeap {
        unsafe { &mut (*(self.addr as *mut MiHeap)) }
    }

    /// init
    /// 需要保证heap_start_addr是4MB对齐，heap_size是4MB的倍数
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn init(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % MIN_SEGMENT_SIZE == 0,
            "Start address should be 4MB aligned"
        );
        assert!(
            heap_size % MIN_SEGMENT_SIZE == 0 && heap_size > 0,
            "Add Heap size should be a multiple of 4MB"
        );
        self.addr = heap_start_addr;
        self.unused_begin = heap_start_addr;
        self.unused_end = heap_start_addr + heap_size;
        self.unused_begin_tmp = 0;
        self.unused_end_tmp = 0;
        self.get_mut_ref().init();
        self.create_small_segment();
    }

    /// 新建一个small类型的segment，并将其中的page塞入heap的free链表中
    /// 从unused_begin中取4MB内存，如果不够则返回false
    pub fn create_small_segment(&mut self) -> bool {
        if self.unused_begin == self.unused_end {
            return false;
        }
        let mut seg_addr = SegmentPointer {
            addr: self.unused_begin,
        };
        let seg = seg_addr.get_mut_ref();
        seg.init(self.unused_begin, MIN_SEGMENT_SIZE, PageKind::Small);
        for i in 0..seg.num_pages {
            let page_addr = PagePointer {
                addr: &seg.pages[i] as *const Page as usize,
            };
            self.get_mut_ref().add_small_page(page_addr);
        }
        self.unused_begin += MIN_SEGMENT_SIZE;
        true
    }

    /// 新建一个medium类型的segment，并将其中的page塞入heap的tmp_page
    /// 从unused_begin中取4MB内存，如果不够则返回false
    pub fn create_medium_segment(&mut self) -> bool {
        if self.unused_begin == self.unused_end {
            return false;
        }
        let mut seg_addr = SegmentPointer {
            addr: self.unused_begin,
        };
        let seg = seg_addr.get_mut_ref();
        seg.init(self.unused_begin, MIN_SEGMENT_SIZE, PageKind::Medium);
        let page_addr = PagePointer {
            addr: &seg.pages[0] as *const Page as usize,
        };
        self.get_mut_ref().add_medium_page(page_addr);
        self.unused_begin += MIN_SEGMENT_SIZE;
        true
    }

    /// 新建一个huge类型的segment，并将其中的page塞入heap的tmp_page
    /// 优先从unused_begin_tmp中取内存
    /// 如果没有再从unused_begin中取内存
    /// 如果还没有则返回false
    pub fn create_huge_segment(&mut self, size: usize) -> bool {
        assert!(
            size % MIN_SEGMENT_SIZE == 0,
            "Huge segment size should be a multiple of 4MB"
        );
        let begin_addr;
        if self.unused_begin_tmp + size <= self.unused_end_tmp {
            begin_addr = self.unused_begin_tmp;
            self.unused_begin_tmp += size;
        } else if self.unused_begin + size <= self.unused_end {
            begin_addr = self.unused_begin;
            self.unused_begin += size;
        } else {
            return false;
        }
        let mut seg_addr = SegmentPointer { addr: begin_addr };
        let seg = seg_addr.get_mut_ref();
        seg.init(begin_addr, size, PageKind::Huge);
        let mut page_addr = PagePointer {
            addr: &seg.pages[0] as *const Page as usize,
        };
        // huge块，事先设定大小
        page_addr
            .get_mut_ref()
            .init_size(size - size_of::<Segment>());
        self.get_mut_ref().insert_to_list(HUGE_QUEUE, page_addr);
        true
    }

    /// Adds memory to the heap. The start address must be valid
    /// 需要保证heap_start_addr是4MB对齐，heap_size是4MB的倍数
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        assert!(
            start_addr % MIN_SEGMENT_SIZE == 0,
            "Start address should be 4MB aligned"
        );
        assert!(
            heap_size % MIN_SEGMENT_SIZE == 0 && heap_size > 0,
            "Add Heap size should be a multiple of 4MB"
        );
        if self.unused_begin == self.unused_end {
            self.unused_begin = start_addr;
            self.unused_end = start_addr + heap_size;
        } else {
            self.unused_begin_tmp = start_addr;
            self.unused_end_tmp = start_addr + heap_size;
        }
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        //单次分配最小8字节
        assert!(
            my_lowbit(layout.align()) == layout.align(),
            "align should be power of 2."
        );
        let align = max(layout.align(), size_of::<usize>());
        // 由于Segment头大小为8192字节，凡是对齐要求小于这个数的分配都可以保证是对齐的
        // 对于超过8192字节对齐，取一个两倍大小的块并返回其中对齐的地址
        let size = (if align > size_of::<Segment>() { 2 } else { 1 })
            * get_upper_size(alignto(layout.size(), align));

        let idx = get_queue_id(size);

        // 找一个page
        // 首先找现成的，如果没有就去找未使用的
        let mut page = self.get_mut_ref().get_page(idx, size);
        // 如果没找到
        if page.addr == 0 {
            let pagetype;
            if size < SMALL_PAGE_SIZE {
                pagetype = PageKind::Small;
            } else if size < MEDIUM_PAGE_SIZE {
                pagetype = PageKind::Medium;
            } else {
                pagetype = PageKind::Huge;
            }

            // 尝试创建一个段，如果创建不了就寄了
            match pagetype {
                PageKind::Small => {
                    if !self.create_small_segment() {
                        return Err(AllocError);
                    }
                }
                PageKind::Medium => {
                    if !self.create_medium_segment() {
                        return Err(AllocError);
                    }
                }
                PageKind::Huge => {
                    if !self
                        .create_huge_segment(alignto(size + size_of::<Segment>(), MIN_SEGMENT_SIZE))
                    {
                        return Err(AllocError);
                    }
                }
            }

            //创建完后再找一次
            page = self.get_mut_ref().get_page(idx, size);
            if page.addr == 0 {
                return Err(AllocError);
            }
        }

        // 获取一个block
        let addr = page.get_mut_ref().get_block();

        // 如果这个块从不满变为满，要塞进full queue里
        if page.get_ref().is_full() {
            self.get_mut_ref().delete_from_list(idx, page);
            self.get_mut_ref().add_full_page(page);
        }

        Ok(if align > size_of::<Segment>() {
            alignto(addr, align)
        } else {
            addr
        })
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn deallocate(&mut self, ptr: usize, layout: Layout) {
        assert!(
            my_lowbit(layout.align()) == layout.align(),
            "align should be power of 2."
        );
        let align = max(layout.align(), size_of::<usize>());
        // 由于Segment头大小为8192字节，凡是对齐要求小于这个数的分配都可以保证是对齐的
        // 对于超过8192字节对齐，取一个两倍大小的块并返回其中对齐的地址
        let size = (if align > size_of::<Segment>() { 2 } else { 1 })
            * get_upper_size(alignto(layout.size(), align));

        let idx = get_queue_id(size);

        let block_pointer = if align > size_of::<Segment>() {
            get_true_block(ptr)
        } else {
            BlockPointer { addr: ptr }
        };

        // 先找到这个块所在的页
        let mut page = get_page(ptr);
        let flag = page.get_ref().is_full();
        page.get_mut_ref().return_block(block_pointer);

        //如果这个块从满变为不满，要塞回原来的queue
        if flag && !page.get_ref().is_full() {
            self.get_mut_ref().del_full_page(page);
            self.get_mut_ref().insert_to_list(idx, page);
        }

        // 如果一个块不是huge，且已经完全空了，就回收
        if size < MEDIUM_PAGE_SIZE && page.get_ref().is_empty() {
            self.get_mut_ref().delete_from_list(idx, page);
            if size < SMALL_PAGE_SIZE {
                self.get_mut_ref().add_small_page(page);
            } else {
                self.get_mut_ref().add_medium_page(page);
            }
        }
    }

    /// get total bytes
    pub fn total_bytes(&self) -> usize {
        0
    }
    /// get used bytes
    pub fn used_bytes(&self) -> usize {
        0
    }
    /// get available bytes
    pub fn available_bytes(&self) -> usize {
        0
    }
}
