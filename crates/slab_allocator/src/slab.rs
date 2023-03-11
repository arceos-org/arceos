use alloc::alloc::{AllocError, Layout};

pub struct Slab {
    block_size: usize,
    free_block_list: FreeBlockList,

    total_blocks: usize,
}

impl Slab {
    pub unsafe fn new(start_addr: usize, slab_size: usize, block_size: usize) -> Slab {
        let num_of_blocks = slab_size / block_size;
        Slab {
            block_size,
            free_block_list: FreeBlockList::new(start_addr, block_size, num_of_blocks),
            total_blocks: num_of_blocks,
        }
    }

    pub fn total_blocks(&self) -> usize {
        self.total_blocks
    }

    pub fn used_blocks(&self) -> usize {
        self.free_block_list.len()
    }

    pub unsafe fn grow(&mut self, start_addr: usize, slab_size: usize) {
        let num_of_blocks = slab_size / self.block_size;
        self.total_blocks += num_of_blocks;
        let mut block_list = FreeBlockList::new(start_addr, self.block_size, num_of_blocks);
        while let Some(block) = block_list.pop() {
            self.free_block_list.push(block);
        }
    }

    pub fn allocate(&mut self, _layout: Layout) -> Result<usize, AllocError> {
        match self.free_block_list.pop() {
            Some(block) => Ok(block.addr()),
            None => Err(AllocError),
        }
    }

    pub fn deallocate(&mut self, ptr: usize) {
        let ptr = ptr as *mut FreeBlock;
        unsafe {
            self.free_block_list.push(&mut *ptr);
        }
    }
}

struct FreeBlockList {
    len: usize,
    head: Option<&'static mut FreeBlock>,
}

impl FreeBlockList {
    unsafe fn new(start_addr: usize, block_size: usize, num_of_blocks: usize) -> FreeBlockList {
        let mut new_list = FreeBlockList::new_empty();
        for i in (0..num_of_blocks).rev() {
            let new_block = (start_addr + i * block_size) as *mut FreeBlock;
            new_list.push(&mut *new_block);
        }
        new_list
    }

    fn new_empty() -> FreeBlockList {
        FreeBlockList { len: 0, head: None }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn pop(&mut self) -> Option<&'static mut FreeBlock> {
        self.head.take().map(|node| {
            self.head = node.next.take();
            self.len -= 1;
            node
        })
    }

    fn push(&mut self, free_block: &'static mut FreeBlock) {
        free_block.next = self.head.take();
        self.len += 1;
        self.head = Some(free_block);
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

impl Drop for FreeBlockList {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

struct FreeBlock {
    next: Option<&'static mut FreeBlock>,
}

impl FreeBlock {
    fn addr(&self) -> usize {
        self as *const _ as usize
    }
}
