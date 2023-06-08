use core::mem::size_of;

/// Block的pointer
#[derive(Clone, Copy)]
pub struct BlockPointer {
    pub addr: usize,
}

impl BlockPointer {
    /// get reference
    pub fn get_ref(&self) -> &Block {
        unsafe { &(*(self.addr as *const Block)) }
    }
    /// get mut reference
    pub fn get_mut_ref(&mut self) -> &mut Block {
        unsafe { &mut (*(self.addr as *mut Block)) }
    }
}

/// page的pointer
#[derive(Clone, Copy)]
pub struct PagePointer {
    pub addr: usize,
}

impl PagePointer {
    /// get reference
    pub fn get_ref(&self) -> &Page {
        unsafe { &(*(self.addr as *const Page)) }
    }
    /// get mut reference
    pub fn get_mut_ref(&mut self) -> &mut Page {
        unsafe { &mut (*(self.addr as *mut Page)) }
    }
}

/// Segment的pointer
#[derive(Clone, Copy)]
pub struct SegmentPointer {
    pub addr: usize,
}

impl SegmentPointer {
    /// get reference
    pub fn get_ref(&self) -> &Segment {
        unsafe { &(*(self.addr as *const Segment)) }
    }
    /// get mut reference
    pub fn get_mut_ref(&mut self) -> &mut Segment {
        unsafe { &mut (*(self.addr as *mut Segment)) }
    }
}

/// mimalloc的一个block
pub struct Block {
    // free链表的下一项指针
    pub next: BlockPointer,
}

/// mimalloc的一个page控制头
#[derive(Clone, Copy)]
pub struct Page {
    // 块大小
    pub block_size: usize,
    // free链表
    pub free_list: BlockPointer,
    // page开始地址
    pub begin_addr: usize,
    // page结束地址
    pub end_addr: usize,
    // 尚未分配过的地址起点
    pub capacity: usize,
    // page链表中的上一项
    pub prev_page: PagePointer,
    // page链表中的下一项
    pub next_page: PagePointer,
    // 剩余块数
    pub free_blocks_num: usize,
}

pub const TOT_QUEUE_NUM: usize = 75;
// >=4MB的页
pub const HUGE_QUEUE: usize = 71;
// 所有满的页
pub const FULL_QUEUE: usize = 72;
// 尚未分配的small page
pub const FREE_SMALL_PAGE_QUEUE: usize = 73;
// 尚未分配的medium page
pub const FREE_MEDIUM_PAGE_QUEUE: usize = 74;

/// mimalloc的heap结构
pub struct MiHeap {
    // page链表
    pub pages: [PagePointer; TOT_QUEUE_NUM],
}

/// lowbit
pub fn my_lowbit(x: usize) -> usize {
    x & ((!x) + 1)
}

/// log2
pub fn my_log2(x: usize) -> usize {
    let mut ans = 0;
    let mut y = x;
    if (y >> 32) > 0 {
        y >>= 32;
        ans += 32;
    }
    if (y >> 16) > 0 {
        y >>= 16;
        ans += 16;
    }
    if (y >> 8) > 0 {
        y >>= 8;
        ans += 8;
    }
    if (y >> 4) > 0 {
        y >>= 4;
        ans += 4;
    }
    if (y >> 2) > 0 {
        y >>= 2;
        ans += 2;
    }
    if (y >> 1) > 0 {
        ans += 1;
    }
    ans
}

/// 获取一个size对齐到align的结果
pub fn alignto(size: usize, align: usize) -> usize {
    (size + align - 1) / align * align
}

/// 获取一个地址对应的队列编号
/// 取8字节对齐之后的高3位
/// 如果超过4MB则返回huge
pub fn get_queue_id(size: usize) -> usize {
    let _size = (size + 7) >> 3;
    // size < 64的情况
    if _size <= 7 {
        return _size - 1;
    }
    // size >= 4MB的情况
    if _size >= (1 << 19) {
        return HUGE_QUEUE;
    }
    let lg = my_log2(_size);
    lg * 4 - 5 + ((_size >> (lg - 2)) & 3)
}

/// 获得向上取整的mimalloc处理的size（8字节对齐，仅含有高3位）
pub fn get_upper_size(size: usize) -> usize {
    let _size = (size + 7) >> 3;
    if _size <= 7 {
        return _size << 3;
    }
    let lg = my_log2(_size);
    let tmp = _size >> (lg - 2);
    if _size == (tmp << (lg - 2)) {
        tmp << (lg + 1)
    } else {
        (tmp + 1) << (lg + 1)
    }
}

/// page的种类，分为3种：SmallPage，MediumPage和HugePage
pub enum PageKind {
    /// small page：大小为64KB，内部块大小<64KB
    Small,
    /// medium page：大小为4MB，内部块大小>=64KB但<4MB
    Medium,
    /// huge page：大小任意大（4MB对齐），内部块大小>=4MB
    Huge,
}

// 每个段最多多少个page
pub const MAX_PAGE_PER_SEGMEGT: usize = 64;
// 每个段的最小大小（4MB）
pub const MIN_SEGMENT_SIZE: usize = 4 * 1024 * 1024;
// small page的大小（64KB）
pub const SMALL_PAGE_SIZE: usize = 64 * 1024;
// medium page的大小（4MB）
pub const MEDIUM_PAGE_SIZE: usize = 4 * 1024 * 1024;

/// mimalloc段结构
/// 同一个段内的page种类是相同的
/// 段地址为4MB对齐
/// SmallPage和MediumPage的段大小为4MB
/// HugePage的段大小任意大
/// medium和huge的一个段里都只有一个page
pub struct Segment {
    // 把mi_heap藏在第一个段的开头
    pub mi_heap: MiHeap,
    // page种类
    pub page_kind: PageKind,
    // 段的大小
    pub size: usize,
    // 包含多少个page
    pub num_pages: usize,
    // 每个page的头结构
    pub pages: [Page; MAX_PAGE_PER_SEGMEGT],
    // padding，使空间对齐到8192
    pub padding: [usize; 434],
    // 接下来就是每个page的实际空间，注意第一个page会小一些
}

impl Page {
    /// init
    pub fn init(&mut self, size: usize, begin_addr: usize, end_addr: usize) {
        self.begin_addr = begin_addr;
        self.end_addr = end_addr;
        self.prev_page = PagePointer { addr: 0 };
        self.next_page = PagePointer { addr: 0 };
        self.init_size(size);
    }

    /// init size
    /// 需要保证当前page是空的
    pub fn init_size(&mut self, size: usize) {
        self.capacity = self.begin_addr;
        self.block_size = size;
        self.free_list = BlockPointer { addr: 0 };
        if self.block_size == 0 {
            self.free_blocks_num = 0;
        } else {
            self.free_blocks_num = (self.end_addr - self.begin_addr) / self.block_size;
        }
    }

    /// 是否已满
    pub fn is_full(&self) -> bool {
        self.free_blocks_num == 0
    }

    /// 是否已空
    pub fn is_empty(&self) -> bool {
        self.block_size != 0
            && self.free_blocks_num == (self.end_addr - self.begin_addr) / self.block_size
    }

    /// 向free链表里插入一项
    pub fn push_front(&mut self, mut block: BlockPointer) {
        block.get_mut_ref().next = self.free_list;
        self.free_list = block;
    }

    /// 把free链表头删除
    pub fn pop_front(&mut self) {
        let mut blk = self.free_list;
        self.free_list = blk.get_ref().next;
        blk.get_mut_ref().next = BlockPointer { addr: 0 };
    }

    /// 找一个block
    pub fn get_block(&mut self) -> usize {
        if self.free_list.addr != 0 {
            let ans = self.free_list;
            self.pop_front();
            self.free_blocks_num -= 1;
            ans.addr
        } else if self.capacity + self.block_size <= self.end_addr {
            let ans = self.capacity;
            self.capacity += self.block_size;
            self.free_blocks_num -= 1;
            ans
        } else {
            0
        }
    }

    /// 归还一个block
    pub fn return_block(&mut self, block: BlockPointer) {
        self.push_front(block);
        self.free_blocks_num += 1;
    }
}

impl Segment {
    /// init
    pub fn init(&mut self, addr: usize, size: usize, page_kind: PageKind) {
        self.page_kind = page_kind;
        self.size = size;
        let page_size = match self.page_kind {
            PageKind::Small => SMALL_PAGE_SIZE,
            PageKind::Medium => MEDIUM_PAGE_SIZE,
            PageKind::Huge => size,
        };
        let mut begin_addr = addr + size_of::<Segment>();
        let mut end_addr = addr + page_size;
        let end = addr + size;
        self.num_pages = 0;
        for i in 0..MAX_PAGE_PER_SEGMEGT {
            if begin_addr == end {
                self.pages[i].init(0, 0, 0);
            } else {
                self.pages[i].init(0, begin_addr, end_addr);
                begin_addr = end_addr;
                end_addr += page_size;
                self.num_pages += 1;
            }
        }
    }
}

impl MiHeap {
    /// init
    pub fn init(&mut self) {
        for i in 0..TOT_QUEUE_NUM {
            self.pages[i] = PagePointer { addr: 0 };
        }
    }
    /// 向链表里插入一个page
    pub fn insert_to_list(&mut self, idx: usize, mut page: PagePointer) {
        if self.pages[idx].addr != 0 {
            let nxt_page = self.pages[idx].get_mut_ref();
            nxt_page.prev_page = page;
        }

        let nw_page = page.get_mut_ref();
        nw_page.prev_page = PagePointer { addr: 0 };
        nw_page.next_page = self.pages[idx];
        self.pages[idx] = page;
    }
    /// 从链表中删除一个page
    pub fn delete_from_list(&mut self, idx: usize, mut page: PagePointer) {
        let nw_page = page.get_mut_ref();
        let mut prv = nw_page.prev_page;
        let mut nxt = nw_page.next_page;
        nw_page.prev_page = PagePointer { addr: 0 };
        nw_page.next_page = PagePointer { addr: 0 };
        if self.pages[idx].addr == page.addr {
            self.pages[idx] = nxt;
        }
        if prv.addr != 0 {
            let prv_ref = prv.get_mut_ref();
            prv_ref.next_page = nxt;
        }
        if nxt.addr != 0 {
            let nxt_ref = nxt.get_mut_ref();
            nxt_ref.prev_page = prv;
        }
    }
    /// 加入一个尚未分配的small page
    pub fn add_small_page(&mut self, page: PagePointer) {
        self.insert_to_list(FREE_SMALL_PAGE_QUEUE, page);
    }
    /// 删去一个尚未分配的small page
    pub fn del_small_page(&mut self, page: PagePointer) {
        self.delete_from_list(FREE_SMALL_PAGE_QUEUE, page);
    }
    /// 加入一个尚未分配的medium page
    pub fn add_medium_page(&mut self, page: PagePointer) {
        self.insert_to_list(FREE_MEDIUM_PAGE_QUEUE, page);
    }
    /// 删去一个尚未分配的medium page
    pub fn del_medium_page(&mut self, page: PagePointer) {
        self.delete_from_list(FREE_MEDIUM_PAGE_QUEUE, page);
    }
    /// 加入一个已满的small page
    pub fn add_full_page(&mut self, page: PagePointer) {
        self.insert_to_list(FULL_QUEUE, page);
    }
    /// 删去一个已满的small page
    pub fn del_full_page(&mut self, page: PagePointer) {
        self.delete_from_list(FULL_QUEUE, page);
    }

    /// 根据一个size和idx获取一个page，找不到则返回0
    pub fn get_page(&mut self, idx: usize, size: usize) -> PagePointer {
        // 如果不是huge，直接取链首就可以
        if idx != HUGE_QUEUE {
            // 如果能找到现成的，就直接取
            if self.pages[idx].addr != 0 {
                self.pages[idx]
            }
            // 否则，尝试从free队列中取
            else if size < SMALL_PAGE_SIZE {
                // small page
                let mut page = self.pages[FREE_SMALL_PAGE_QUEUE];
                if page.addr != 0 {
                    self.del_small_page(page);
                    page.get_mut_ref().init_size(size);
                    self.insert_to_list(idx, page);
                }
                page
            } else {
                // medium page
                let mut page = self.pages[FREE_MEDIUM_PAGE_QUEUE];
                if page.addr != 0 {
                    self.del_medium_page(page);
                    page.get_mut_ref().init_size(size);
                    self.insert_to_list(idx, page);
                }
                page
            }
        } else {
            let mut fit_size = 0;
            // 遍历所有的huge page，找一个大小最合适的
            let mut page = self.pages[idx];
            while page.addr != 0 {
                let _size = page.get_mut_ref().block_size;
                if _size >= size && (fit_size == 0 || _size < fit_size) {
                    fit_size = _size;
                }
                page = page.get_mut_ref().next_page;
            }

            // 找不到满足要求的page
            if fit_size == 0 {
                return PagePointer { addr: 0 };
            }

            page = self.pages[idx];
            while page.addr != 0 {
                if page.get_mut_ref().block_size == fit_size {
                    return page;
                }
                page = page.get_mut_ref().next_page;
            }
            PagePointer { addr: 0 }
        }
    }
}

/// 找到一个addr的段
/// 就是将地址向下取整到4MB
pub fn get_segment(addr: usize) -> SegmentPointer {
    SegmentPointer {
        addr: (addr / MIN_SEGMENT_SIZE) * MIN_SEGMENT_SIZE,
    }
}

/// 找到一个addr的page
/// 就是先找到段之后，根据段中page的大小来确定
pub fn get_page(addr: usize) -> PagePointer {
    // 先找到段
    let seg = get_segment(addr);
    let page_size = match seg.get_ref().page_kind {
        PageKind::Small => SMALL_PAGE_SIZE,
        PageKind::Medium => MEDIUM_PAGE_SIZE,
        PageKind::Huge => seg.get_ref().size,
    };
    // 再找到page的编号
    let idx = (addr - seg.addr) / page_size;
    PagePointer {
        addr: &seg.get_ref().pages[idx] as *const Page as usize,
    }
}

/// 已知一个addr在某个block里，找到这个block的真实起始地址
/// 可用于aligned alloc
pub fn get_true_block(addr: usize) -> BlockPointer {
    // 先找到段
    let seg = get_segment(addr);
    let page_size = match seg.get_ref().page_kind {
        PageKind::Small => SMALL_PAGE_SIZE,
        PageKind::Medium => MEDIUM_PAGE_SIZE,
        PageKind::Huge => seg.get_ref().size,
    };
    // 再找到page的编号
    let idx = (addr - seg.addr) / page_size;
    // block的真实大小
    let block_size = seg.get_ref().pages[idx].block_size;
    // page的起始地址
    let page_begin = seg.addr
        + if idx == 0 {
            size_of::<Segment>()
        } else {
            idx * page_size
        };
    BlockPointer {
        addr: page_begin + (addr - page_begin) / block_size * block_size,
    }
}
