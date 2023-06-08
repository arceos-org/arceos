use core::mem::size_of;

/// the address pointer
#[derive(Clone, Copy)]
pub struct AddrPointer {
    pub addr: usize,
}

impl AddrPointer {
    pub fn get_block_header(&self) -> &BlockHeader {
        unsafe { &(*(self.addr as *const BlockHeader)) }
    }
    pub fn get_mut_block_header(&mut self) -> &mut BlockHeader {
        unsafe { &mut (*(self.addr as *mut BlockHeader)) }
    }
    pub fn get_controller(&self) -> &Controller {
        unsafe { &(*(self.addr as *const Controller)) }
    }
    pub fn get_mut_controller(&mut self) -> &mut Controller {
        unsafe { &mut (*(self.addr as *mut Controller)) }
    }

    /// BlockHeader的功能
    pub fn get_prev_phy_pointer(&self) -> AddrPointer {
        return self.get_block_header().prev_phy;
    }
    pub fn set_prev_phy_pointer(&mut self, prev_phy: AddrPointer) {
        self.get_mut_block_header().prev_phy = prev_phy;
    }
    pub fn get_prev_free_pointer(&self) -> AddrPointer {
        return self.get_block_header().prev_free;
    }
    pub fn set_prev_free_pointer(&mut self, prev_free: AddrPointer) {
        self.get_mut_block_header().prev_free = prev_free;
    }
    pub fn get_next_free_pointer(&self) -> AddrPointer {
        return self.get_block_header().next_free;
    }
    pub fn set_next_free_pointer(&mut self, next_free: AddrPointer) {
        self.get_mut_block_header().next_free = next_free;
    }

    pub fn get_now_free(&self) -> bool {
        return self.get_block_header().get_now_free();
    }
    pub fn get_prev_free(&self) -> bool {
        return self.get_block_header().get_prev_free();
    }
    pub fn set_now_free(&mut self) {
        self.get_mut_block_header().set_now_free();
    }
    pub fn set_now_used(&mut self) {
        self.get_mut_block_header().set_now_used();
    }
    pub fn set_prev_free(&mut self) {
        self.get_mut_block_header().set_prev_free();
    }
    pub fn set_prev_used(&mut self) {
        self.get_mut_block_header().set_prev_used();
    }

    ///设置以及判断一个块是否为null
    pub fn set_null(&mut self) {
        self.get_mut_block_header().set_null();
    }
    pub fn is_null(&self) -> bool {
        return self.get_block_header().is_null();
    }

    pub fn get_size(&self) -> usize {
        return self.get_block_header().get_size();
    }
    pub fn set_size(&mut self, size: usize) {
        self.get_mut_block_header().set_size(size);
    }

    ///设置这个块为used，除了要设置自己还要设置物理上的下一个块
    pub fn set_used(&mut self) {
        self.get_mut_block_header().set_used();
    }
    ///设置这个块为free，除了要设置自己还要设置物理上的下一个块
    pub fn set_free(&mut self) {
        self.get_mut_block_header().set_free();
    }

    ///Controller的功能
    pub fn init_controller(&mut self, addr: usize, size: usize) {
        self.get_mut_controller().init(addr, size);
    }
    pub fn add_memory(&mut self, addr: usize, size: usize) {
        self.get_mut_controller().add_memory(addr, size);
    }
    pub fn add_into_list(&mut self, block: AddrPointer) {
        self.get_mut_controller().add_into_list(block);
    }
    pub fn del_into_list(&mut self, block: AddrPointer) {
        self.get_mut_controller().del_into_list(block);
    }
    pub fn get_first_block(&mut self, fl: usize, sl: usize) -> AddrPointer {
        self.get_mut_controller().get_first_block(fl, sl)
    }
    pub fn find_block(&mut self, size: usize) -> AddrPointer {
        self.get_mut_controller().find_block(size)
    }
}

pub fn get_addr_pointer(addr: usize) -> AddrPointer {
    AddrPointer { addr }
}

/// tlsf块头结构
pub struct BlockHeader {
    pub prev_phy: AddrPointer, //物理上的上一个块
    pub size: usize, //“净”块大小，一定是8对齐的，所以末两位可以标记物理上的这个块/上一个块是否free
    //以上16个字节是分配出去的块要占的头部大小
    pub prev_free: AddrPointer, //free链表的上一个块
    pub next_free: AddrPointer, //free链表的下一个块
}

impl BlockHeader {
    pub fn get_now_free(&self) -> bool {
        (self.size & 1) == 1
    }
    pub fn get_prev_free(&self) -> bool {
        (self.size & 2) == 2
    }
    pub fn set_now_free(&mut self) {
        self.size |= 1;
    }
    pub fn set_now_used(&mut self) {
        self.size &= !1_usize;
    }
    pub fn set_prev_free(&mut self) {
        self.size |= 2;
    }
    pub fn set_prev_used(&mut self) {
        self.size &= !2_usize;
    }

    pub fn get_my_pointer(&self) -> AddrPointer {
        AddrPointer {
            addr: self as *const BlockHeader as usize,
        }
    }

    ///设置以及判断一个块是否为null
    pub fn set_null(&mut self) {
        self.size = 0;
        let my_pointer = self.get_my_pointer();
        self.prev_free = my_pointer;
        self.next_free = my_pointer;
        self.prev_phy = my_pointer;
    }
    pub fn is_null(&self) -> bool {
        self.size < 4
    }

    pub fn get_size(&self) -> usize {
        self.size & (!3_usize)
    }
    pub fn set_size(&mut self, size: usize) {
        self.size = size | (self.size & 3);
    }

    ///设置这个块为used，除了要设置自己还要设置物理上的下一个块
    pub fn set_used(&mut self) {
        let mut next = get_block_phy_next(self.get_my_pointer());
        self.set_now_used();
        if !(next.is_null()) {
            next.set_prev_used();
        }
    }
    ///设置这个块为free，除了要设置自己还要设置物理上的下一个块
    pub fn set_free(&mut self) {
        let mut next = get_block_phy_next(self.get_my_pointer());
        self.set_now_free();
        if !(next.is_null()) {
            next.set_prev_free();
        }
    }
}

///获取一个块物理上的下一个块
pub fn get_block_phy_next(block: AddrPointer) -> AddrPointer {
    AddrPointer {
        addr: (block.addr + block.get_size() + 2 * size_of::<usize>()),
    }
}
///获取一个块物理上的上一个块
pub fn get_block_phy_prev(block: AddrPointer) -> AddrPointer {
    block.get_prev_phy_pointer()
}

const FL_INDEX_COUNT: usize = 28;
const SL_INDEX_COUNT: usize = 32;
const FL_INDEX_SHIFT: usize = 8;
const SMALL_BLOCK_SIZE: usize = 256;
//地址的后3位一定是0
//对于不足256的块，直接8对齐
//对于超过256的块，最高位表示一级链表，接下来5位表示二级链表

/// tlsf 控制头结构
pub struct Controller {
    pub block_null: BlockHeader, //空块

    /* Bitmaps for free lists. */
    pub fl_bitmap: usize, //一级链表的bitmap，标记每个一级链表是否非空
    pub sl_bitmap: [usize; FL_INDEX_COUNT], //二级链表的bitmap，标记每个二级链表是否非空

    /* Head of free lists. */
    pub blocks: [[AddrPointer; SL_INDEX_COUNT]; FL_INDEX_COUNT], //二级链表结构
                                                                 //SL_INDEX_COUNT=32表示二级链表将一级链表的一个区间拆分成了32段，也就是要根据最高位后的5个二进制位来判断
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

/// log lowbit
pub fn my_log_lowbit(x: usize) -> usize {
    my_log2(my_lowbit(x))
}

/// 获取一个size对齐到align的结果
pub fn alignto(size: usize, align: usize) -> usize {
    (size + align - 1) / align * align
}

pub struct ListIndex {
    pub fl: usize,
    pub sl: usize,
    pub size: usize,
}
/// 获取一个块对应的一级和二级链表
pub fn get_fl_and_sl(size: usize) -> ListIndex {
    if size < SMALL_BLOCK_SIZE {
        //小块
        ListIndex {
            fl: 0,
            sl: size >> 3,
            size,
        }
    } else {
        let tmp = (my_log2(size)) - FL_INDEX_SHIFT + 1;
        ListIndex {
            fl: tmp,
            sl: (size >> (tmp + 2)) & (SL_INDEX_COUNT - 1),
            size,
        }
    }
}

/// 给定二级链表，求其最小块大小
pub fn get_block_begin_size(fl: usize, sl: usize) -> usize {
    if fl == 0 {
        sl << 3
    } else {
        (1_usize << (fl + FL_INDEX_SHIFT - 1)) + (sl << (fl + 2))
    }
}

/// 获取一个块向上对齐到一级链表的最小块大小
pub fn get_up_size(size: usize) -> usize {
    let mut nsize = size;
    if size < SMALL_BLOCK_SIZE {
        //小块
        alignto(size, 8)
    } else {
        let linkidx = get_fl_and_sl(size);
        let fl = linkidx.fl;
        let sl = linkidx.sl;
        if get_block_begin_size(fl, sl) != size {
            nsize += 1_usize << (fl + 2);
        }
        nsize
    }
}

/// 获取一个块向上对齐到一级链表的最小块大小，以及相应的fl和sl
fn get_up_fl_and_sl(size: usize) -> ListIndex {
    let nsize = get_up_size(size);
    get_fl_and_sl(nsize)
}

impl Controller {
    ///init
    pub fn init(&mut self, addr: usize, size: usize) {
        self.block_null.set_null();
        self.fl_bitmap = 0;
        for i in 0..FL_INDEX_COUNT {
            self.sl_bitmap[i] = 0;
        }
        for i in 0..FL_INDEX_COUNT {
            for j in 0..SL_INDEX_COUNT {
                self.blocks[i][j] = self.block_null.get_my_pointer();
            }
        }
        // 把剩余的空间用于添加内存
        let offset = alignto(size_of::<Controller>(), 8);
        self.add_memory(addr + offset, size - offset);
    }

    /// add memory
    /// addr和size都应该是8对齐的
    pub fn add_memory(&mut self, addr: usize, size: usize) {
        //第一个块
        let mut first = get_addr_pointer(addr);
        let null_pointer = self.block_null.get_my_pointer();
        first.set_prev_phy_pointer(null_pointer);
        first.set_next_free_pointer(null_pointer);
        first.set_prev_free_pointer(null_pointer);
        //set_size传入是这个块的“净大小”，要扣去头部的16个字节和尾部多一个null块的32字节
        first.set_now_free();
        first.set_prev_used();
        first.set_size(size - 6 * size_of::<usize>());
        //把第一个块插入到链表中
        self.add_into_list(first);
        //尾部再加一个null块，占32字节
        let mut tail = get_addr_pointer(addr + size - 4 * size_of::<usize>());
        tail.set_null();
    }

    /// 把一个块插入free list中，需要确保这个块是空闲的
    pub fn add_into_list(&mut self, mut block: AddrPointer) {
        let size = block.get_size();
        let listidx = get_fl_and_sl(size);
        let fl = listidx.fl;
        let sl = listidx.sl;
        //获取了这个块的二级链表之后，插入
        let mut head = self.blocks[fl][sl];
        block.set_next_free_pointer(head);
        block.set_prev_free_pointer(self.block_null.get_my_pointer());
        if !(head.is_null()) {
            head.set_prev_free_pointer(block);
        }
        // 别忘了修改链表头以及修改bitmap
        self.blocks[fl][sl] = block;
        self.sl_bitmap[fl] |= 1_usize << sl;
        self.fl_bitmap |= 1_usize << fl;
    }

    ///把一个块从list中删除，需要确保它之前确实在free list里
    pub fn del_into_list(&mut self, block: AddrPointer) {
        let size = block.get_size();
        let listidx = get_fl_and_sl(size);
        let fl = listidx.fl;
        let sl = listidx.sl;
        let mut prev = block.get_prev_free_pointer();
        let mut next = block.get_next_free_pointer();
        if !(prev.is_null()) {
            prev.set_next_free_pointer(next);
        } else {
            //要更新链表头
            self.blocks[fl][sl] = next;
            if next.is_null() {
                //要更新bitmap
                self.sl_bitmap[fl] &= !(1_usize << sl);
                if self.sl_bitmap[fl] == 0 {
                    self.fl_bitmap &= !(1_usize << fl);
                }
            }
        }
        if !(next.is_null()) {
            next.set_prev_free_pointer(prev);
        }
    }

    /// 获取某一个链表的第一个块，并从链表中删除
    pub fn get_first_block(&mut self, fl: usize, sl: usize) -> AddrPointer {
        if self.blocks[fl][sl].is_null() {
            return self.block_null.get_my_pointer();
        }
        let block = self.blocks[fl][sl];
        self.del_into_list(block);
        block
    }

    /// 给定大小，获取一个能用的块，并从链表中删除
    pub fn find_block(&mut self, size: usize) -> AddrPointer {
        let listidx = get_up_fl_and_sl(size);
        let mut fl = listidx.fl;
        let mut sl = listidx.sl;
        let psl = !((1_usize << sl) - 1); //第二级链表的掩码
        if (psl & self.sl_bitmap[fl]) != 0 {
            //可以在当前一级链表里找到块
            sl = my_log_lowbit(psl & self.sl_bitmap[fl]);
        } else {
            let pfl = !((1_usize << (fl + 1)) - 1); //第一级链表的掩码
            if (pfl & self.fl_bitmap) != 0 {
                //可以在更高的一级链表里找到块
                fl = my_log_lowbit(pfl & self.fl_bitmap);
                sl = my_log_lowbit(self.sl_bitmap[fl]);
            } else {
                return self.block_null.get_my_pointer();
            }
        }
        self.get_first_block(fl, sl)
    }
}
