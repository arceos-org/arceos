//! TLSF allocator for `no_std` systems.
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
pub struct HeapRust {
    head: AddrPointer, // 指向TLSF头结构的指针
    total_mem: usize,  // 总共占用内存
    used_mem: usize,   // 已经分配出去的内存
    avail_mem: usize,  // 实际可用的内存
}

unsafe impl Send for HeapRust {}

impl HeapRust {
    /// Create an empty heap
    pub const fn new() -> Self {
        HeapRust {
            head: AddrPointer { addr: 0 },
            total_mem: 0,
            used_mem: 0,
            avail_mem: 0,
        }
    }

    ///init
    pub fn init(&mut self, heap_start_addr: usize, heap_size: usize) {
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.head = get_addr_pointer(heap_start_addr);
        self.head.init_controller(heap_start_addr, heap_size);
        self.total_mem = heap_size;
        self.used_mem = 0;
        self.avail_mem = heap_size - alignto(size_of::<Controller>(), 8) - 6 * size_of::<usize>();
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        assert!(
            start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.head.add_memory(start_addr, heap_size);
        self.total_mem += heap_size;
        self.avail_mem += heap_size - 6 * size_of::<usize>();
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        //单次分配最小16字节
        assert!(
            my_lowbit(layout.align()) == layout.align(),
            "align should be power of 2."
        );
        let mut size = alignto(
            max(layout.size(), max(layout.align(), 2 * size_of::<usize>())),
            max(layout.align(), size_of::<usize>()),
        );

        //处理align更大的分配请求
        if layout.align() > size_of::<usize>() {
            size = alignto(
                size + layout.align() + 4 * size_of::<usize>(),
                layout.align(),
            );
            //给size加上足够的大小，使得切出来的块的头部可以分裂成一个新的块
        }

        let mut block = self.head.find_block(size);
        if !(block.is_null()) {
            let mut nsize = block.get_size();
            assert!(nsize >= size, "Alloc error.");
            let mut addr = block.addr + 2 * size_of::<usize>();

            //处理align更大的分配请求
            if layout.align() > size_of::<usize>() {
                let mut new_addr = alignto(addr, layout.align());
                if new_addr != addr {
                    //要切出头部单独组成一块
                    while new_addr - block.addr < 6 * size_of::<usize>() {
                        //切出的头部不足以构成一个新块，于是把头部再扩大一个align
                        //因为new_addr是实际分配出去的起始地址，因此到原来块的开头至少要48个字节才能让中间再拆出一个块
                        new_addr += layout.align();
                    }
                    //创造一个新的块pre_block
                    let mut pre_block = block;
                    let mut nxt_block = get_block_phy_next(block);
                    block = get_addr_pointer(new_addr - 2 * size_of::<usize>());
                    //设置物理上的前一块
                    block.set_prev_phy_pointer(pre_block);
                    if !(nxt_block.is_null()) {
                        nxt_block.set_prev_phy_pointer(block);
                    }
                    //设置块大小
                    let pre_size = block.addr - addr;
                    nsize -= pre_size + 2 * size_of::<usize>();
                    pre_block.set_size(pre_size);
                    block.set_size(nsize);
                    //设置使用状态
                    pre_block.set_free();
                    //插回到链表中去
                    self.head.add_into_list(pre_block);
                    self.avail_mem -= 2 * size_of::<usize>();
                    addr = new_addr;
                }

                //把size改回来，这里的size就是实际分配出去的大小了
                size = alignto(
                    max(layout.size(), max(layout.align(), 2 * size_of::<usize>())),
                    layout.align(),
                );
                assert!(nsize >= size, "Alloc error.");
            }
            block.set_used();

            //把块的尾部拆分之后扔回去
            if nsize >= size + 4 * size_of::<usize>() {
                //最小32字节才能切出一个新块
                //新块
                let mut new_block = get_addr_pointer(addr + size);
                new_block.set_prev_phy_pointer(block);
                //原块的下一个块
                let mut nxt_block = get_block_phy_next(block);
                if !(nxt_block.is_null()) {
                    nxt_block.set_prev_phy_pointer(new_block);
                }
                //设置块大小
                block.set_size(size);
                new_block.set_size(nsize - size - 2 * size_of::<usize>()); //别忘了减去新块的头部大小
                                                                           //设置使用状态
                block.set_used();
                new_block.set_free();
                //插回到链表中去
                self.head.add_into_list(new_block);
                self.avail_mem -= 2 * size_of::<usize>();
            }
            self.used_mem += layout.size();
            self.avail_mem -= block.get_size();
            Ok(addr)
        } else {
            Err(AllocError)
        }
    }

    /// 把这个块和物理上后一个块合并，要求两个块都是空闲的，且已经从链表中摘下来了
    pub fn merge_block(&self, mut block: AddrPointer) {
        let nxt = get_block_phy_next(block);
        //改block的size
        let size = block.get_size();
        let nsize = nxt.get_size();
        block.set_size(size + nsize + 2 * size_of::<usize>());
        //改block.nxt.nxt的pre指针为block自己
        let mut nnxt = get_block_phy_next(nxt);
        if !(nnxt.is_null()) {
            nnxt.set_prev_phy_pointer(block);
        }
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
        let size = alignto(
            max(layout.size(), max(layout.align(), 2 * size_of::<usize>())),
            max(layout.align(), size_of::<usize>()),
        );
        let mut block = get_addr_pointer(ptr - 2 * size_of::<usize>());
        let block_size = block.get_size();
        assert!(block_size >= size && !block.get_now_free(), "Dealloc error");
        block.set_free();
        self.used_mem -= layout.size();
        self.avail_mem += block_size;

        //把这个块与前后的块合并
        let mut nblock = block;
        let pre = get_block_phy_prev(block);
        let nxt = get_block_phy_next(block);
        if !(nxt.is_null()) && nxt.get_now_free() {
            //如果物理上的下一个块不是null且是空闲的，就合并
            self.head.del_into_list(nxt);
            self.merge_block(nblock);
            self.avail_mem += 2 * size_of::<usize>();
        }
        if !pre.is_null() && pre.get_now_free() {
            //如果物理上的上一个块不是null且是空闲的，就合并
            self.head.del_into_list(pre);
            self.merge_block(pre);
            nblock = pre;
            self.avail_mem += 2 * size_of::<usize>();
        }
        self.head.add_into_list(nblock);
    }

    /// get total bytes
    pub fn total_bytes(&self) -> usize {
        self.total_mem
    }
    /// get used bytes
    pub fn used_bytes(&self) -> usize {
        self.used_mem
    }
    /// get available bytes
    pub fn available_bytes(&self) -> usize {
        self.avail_mem
    }
}

use core::ffi::c_ulonglong;
#[link(name = "tlsf")]
extern "C" {
    /// create the TLSF structure with a memory pool
    pub fn tlsf_create_with_pool(mem: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;
    /// add a memory pool to existing TLSF structure
    pub fn tlsf_add_pool(tlsf: c_ulonglong, mem: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;
    /// malloc
    pub fn tlsf_malloc(tlsf: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;
    /// malloc an aligned memory
    pub fn tlsf_memalign(tlsf: c_ulonglong, align: c_ulonglong, bytes: c_ulonglong) -> c_ulonglong;
    /// free
    pub fn tlsf_free(tlsf: c_ulonglong, ptr: c_ulonglong);
}

/// the inner heap of TLSF_C Allocator
pub struct HeapC {
    inner: Option<c_ulonglong>,
}

impl HeapC {
    /// create a new heap
    pub const fn new() -> Self {
        Self { inner: None }
    }
    /// get inner mut
    pub fn inner_mut(&mut self) -> &mut c_ulonglong {
        self.inner.as_mut().unwrap()
    }
    /// get inner
    pub fn inner(&self) -> &c_ulonglong {
        self.inner.as_ref().unwrap()
    }
    /// init
    pub fn init(&mut self, start: usize, size: usize) {
        unsafe {
            self.inner = Some(
                tlsf_create_with_pool(start as c_ulonglong, size as c_ulonglong) as c_ulonglong,
            );
        }
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn add_memory(&mut self, start: usize, size: usize) {
        unsafe {
            tlsf_add_pool(
                *self.inner() as c_ulonglong,
                start as c_ulonglong,
                size as c_ulonglong,
            );
        }
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    pub fn allocate(&mut self, size: usize, align_pow2: usize) -> Result<usize, AllocError> {
        if align_pow2 <= 8 {
            unsafe {
                let ptr = tlsf_malloc(*self.inner() as c_ulonglong, size as c_ulonglong) as usize;
                if ptr == 0 {
                    return Err(AllocError);
                }
                Ok(ptr)
            }
        } else {
            unsafe {
                let ptr = tlsf_memalign(
                    *self.inner() as c_ulonglong,
                    align_pow2 as c_ulonglong,
                    size as c_ulonglong,
                ) as usize;
                if ptr == 0 {
                    return Err(AllocError);
                }
                Ok(ptr)
            }
        }
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn deallocate(&mut self, pos: usize, _size: usize, _align_pow2: usize) {
        unsafe {
            tlsf_free(*self.inner() as c_ulonglong, pos as c_ulonglong);
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
