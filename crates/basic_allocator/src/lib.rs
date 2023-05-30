#![feature(allocator_api)]
#![no_std]

extern crate alloc;

use alloc::alloc::{AllocError, Layout};
use alloc::vec::Vec;
use core::mem::size_of;
pub mod linked_list;
use core::cmp::max;
pub use linked_list::{LinkedList, MemBlockFoot, MemBlockHead};

pub enum BasicAllocatorStrategy {
    FirstFitStrategy,
    BestFitStrategy,
    WorstFitStrategy,
}

pub struct Heap {
    free_list: LinkedList,
    user: usize,                      //分配给用户的内存大小
    allocated: usize,                 //实际分配出去的内存大小
    total: usize,                     //总内存大小
    strategy: BasicAllocatorStrategy, //使用的内存分配策略
    begin_addr: usize,                //堆区起始地址
    end_addr: usize,                  //堆区结束地址

    //处理kernel page table，对此的申请是不经过这里的，这会形如在堆空间中挖了一个洞
    kernel_begin: Vec<usize>,
    kernel_end: Vec<usize>,
}

/// 获取一个地址加上一个usize(分配出去的块的头)大小后对齐到align的结果
fn get_aligned(addr: usize, align: usize) -> usize {
    (addr + size_of::<usize>() + align - 1) / align * align
}

/// 获取一个size对齐到align的结果
fn alignto(size: usize, align: usize) -> usize {
    (size + align - 1) / align * align
}

impl Heap {
    /// Create an empty heap
    pub const fn new() -> Self {
        Heap {
            free_list: LinkedList::new(),
            user: 0,
            allocated: 0,
            total: 0,

            //strategy: BasicAllocatorStrategy::FirstFitStrategy,
            strategy: BasicAllocatorStrategy::BestFitStrategy,
            //strategy: BasicAllocatorStrategy::WorstFitStrategy,
            begin_addr: 0,
            end_addr: 0,

            kernel_begin: Vec::new(),
            kernel_end: Vec::new(),
        }
    }

    ///init
    pub fn init(&mut self, heap_start_addr: usize, heap_size: usize, strategy: &str) {
        assert!(
            heap_start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.free_list = LinkedList::new();
        self.kernel_begin = Vec::new();
        self.kernel_end = Vec::new();
        self.begin_addr = heap_start_addr;
        self.end_addr = heap_start_addr + heap_size;
        self.push_mem_block(heap_start_addr, heap_size);
        self.total = heap_size;
        self.set_strategy(strategy);
    }

    ///set strategy
    pub fn set_strategy(&mut self, strategy: &str) {
        match strategy {
            "first_fit" => {
                self.strategy = BasicAllocatorStrategy::FirstFitStrategy;
            }
            "best_fit" => {
                self.strategy = BasicAllocatorStrategy::BestFitStrategy;
            }
            "worst_fit" => {
                self.strategy = BasicAllocatorStrategy::WorstFitStrategy;
            }
            _ => {
                panic!("unknown basic alloc strategy!");
            }
        }
    }

    /// Adds memory to the heap. The start address must be valid
    /// and the memory in the `[mem_start_addr, mem_start_addr + heap_size)` range must not be used for
    /// anything else.
    /// In case of linked list allocator the memory can only be extended.
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub fn add_memory(&mut self, start_addr: usize, heap_size: usize) {
        if start_addr != self.end_addr {
            self.kernel_begin.push(self.end_addr);
            self.kernel_end.push(start_addr);
        }
        assert!(
            start_addr % 4096 == 0,
            "Start address should be page aligned"
        );
        assert!(
            heap_size % 4096 == 0 && heap_size > 0,
            "Add Heap size should be a multiple of page size"
        );
        self.end_addr = start_addr + heap_size;
        self.push_mem_block(start_addr, heap_size);
        self.total += heap_size;
    }

    /// fitst fit策略
    pub fn first_fit(&mut self, size: usize, align: usize) -> Option<*mut MemBlockHead> {
        let mut block = self.free_list.head;
        while !block.is_null() {
            unsafe {
                let addr = block as usize;
                let bsize = (*block).size();
                if addr + bsize >= get_aligned(addr, align) + size + size_of::<usize>() {
                    return Some(block);
                }
                block = (*block).nxt;
            }
        }
        None
    }

    /// best fit策略
    pub fn best_fit(&mut self, size: usize, align: usize) -> Option<*mut MemBlockHead> {
        let mut res: Option<*mut MemBlockHead> = None;
        let mut now_size: usize = 0;
        let mut block = self.free_list.head;
        while !block.is_null() {
            unsafe {
                let addr = block as usize;
                let bsize = (*block).size();
                if addr + bsize >= get_aligned(addr, align) + size + size_of::<usize>() {
                    let addr_left =
                        addr + bsize - get_aligned(addr, align) - size - size_of::<usize>();
                    if res.is_none() || addr_left < now_size {
                        now_size = addr_left;
                        res = Some(block);
                    }
                }
                block = (*block).nxt;
            }
        }
        res
    }

    /// worst fit策略
    pub fn worst_fit(&mut self, size: usize, align: usize) -> Option<*mut MemBlockHead> {
        let mut res: Option<*mut MemBlockHead> = None;
        let mut now_size: usize = 0;
        let mut block = self.free_list.head;
        while !block.is_null() {
            unsafe {
                let addr = block as usize;
                let bsize = (*block).size();
                if addr + bsize >= get_aligned(addr, align) + size + size_of::<usize>() {
                    let addr_left =
                        addr + bsize - get_aligned(addr, align) - size - size_of::<usize>();
                    if res.is_none() || addr_left > now_size {
                        now_size = addr_left;
                        res = Some(block);
                    }
                }
                block = (*block).nxt;
            }
        }
        res
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `Err`.
    pub fn allocate(&mut self, layout: Layout) -> Result<usize, AllocError> {
        //单次分配最小16字节
        let size = alignto(
            max(layout.size(), max(layout.align(), 2 * size_of::<usize>())),
            size_of::<usize>(),
        );

        unsafe {
            let block = match self.strategy {
                BasicAllocatorStrategy::FirstFitStrategy => self.first_fit(size, layout.align()),
                BasicAllocatorStrategy::BestFitStrategy => self.best_fit(size, layout.align()),
                BasicAllocatorStrategy::WorstFitStrategy => self.worst_fit(size, layout.align()),
            };
            match block {
                Some(inner) => {
                    let res = inner as usize;
                    let block_size = (*inner).size();
                    //地址对齐
                    let addr = get_aligned(res, layout.align());
                    let addr_left = res + block_size - addr - size - size_of::<usize>();
                    if addr_left > 4 * size_of::<usize>() {
                        //还能切出去更小的块
                        (*inner).set_size(block_size - addr_left);
                        (*inner).set_used(true);
                        self.free_list.del(inner);
                        self.user += layout.size();
                        self.allocated += block_size - addr_left;
                        //一定不会merge
                        self.free_list.push(
                            (addr + size + size_of::<usize>()) as *mut MemBlockHead,
                            addr_left,
                        );
                    } else {
                        (*inner).set_used(true);
                        self.user += layout.size();
                        self.allocated += block_size;
                        self.free_list.del(inner);
                    }

                    Ok(addr)
                }
                None => Err(AllocError),
            }
        }
    }

    /// push a memblock to linked list
    /// before push,need to check merge
    pub fn push_mem_block(&mut self, addr: usize, size: usize) {
        let mut now_addr = addr;
        let mut now_size = size;

        //先找:是否有一个块紧贴在它后面
        if now_addr + now_size != self.end_addr
            && !self.kernel_begin.contains(&(now_addr + now_size))
        {
            let nxt_block = (now_addr + now_size) as *mut MemBlockHead;
            unsafe {
                if !(*nxt_block).used() {
                    now_size += (*nxt_block).size();
                    self.free_list.del(nxt_block);
                }
            }
        }
        //再找:是否有一个块紧贴在它前面
        if now_addr != self.begin_addr && !self.kernel_end.contains(&(now_addr)) {
            unsafe {
                let pre_block =
                    (*((now_addr - size_of::<usize>()) as *mut MemBlockFoot)).get_head();
                if !(*pre_block).used() {
                    now_addr = pre_block as usize;
                    now_size += (*pre_block).size();
                    self.free_list.del(pre_block);
                }
            }
        }
        unsafe {
            self.free_list.push(now_addr as *mut MemBlockHead, now_size);
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
        let size = alignto(
            max(layout.size(), max(layout.align(), 2 * size_of::<usize>())),
            size_of::<usize>(),
        );
        let block = (ptr - size_of::<usize>()) as *mut MemBlockHead;
        unsafe {
            let block_size = (*block).size();
            assert!(block_size >= size, "Dealloc error");
            self.user -= layout.size();
            self.allocated -= block_size;
            self.push_mem_block(block as usize, block_size);
        }
    }

    pub fn total_bytes(&self) -> usize {
        self.total
    }

    pub fn used_bytes(&self) -> usize {
        self.user
    }

    pub fn available_bytes(&self) -> usize {
        self.total - self.allocated
    }
}
