//! Provide the intrusive LinkedList for basic allocator
#![allow(dead_code)]

use core::mem::size_of;
use core::ptr;

/// An intrusive linked list
///
/// A clean room implementation of the one used in CS140e 2018 Winter
///
/// Thanks Sergio Benitez for his excellent work,
/// See [CS140e](https://cs140e.sergio.bz/) for more information

pub struct MemBlockHead {
    pub size: usize, //最低位标记是否使用过
    pub pre: *mut MemBlockHead,
    pub nxt: *mut MemBlockHead,
}

impl MemBlockHead {
    ///获取这一块的地址
    pub fn addr(&mut self) -> usize {
        self as *mut MemBlockHead as usize
    }
    //获取这一块的大小
    pub fn size(&self) -> usize {
        self.size >> 1
    }
    ///获取这一块的使用位
    pub fn used(&self) -> bool {
        (self.size & 1) == 1
    }

    ///重设这一块的大小，别忘了同时修改foot的信息
    ///在同时需要修改size和used的时候，一定注意要先改size再改used
    pub fn set_size(&mut self, size: usize) {
        self.size = (size << 1) | (self.size & 1);
        unsafe { (*(self.get_foot())).set_size(size) };
    }
    ///重设这一块的使用位，别忘了同时修改foot的信息
    pub fn set_used(&mut self, used: bool) {
        if used {
            self.size |= 1_usize;
        } else {
            self.size &= !1_usize;
        }
        unsafe {
            (*(self.get_foot())).set_used(used);
        }
    }

    ///获取这一块的foot
    pub fn get_foot(&mut self) -> *mut MemBlockFoot {
        (self.addr() + self.size() - size_of::<usize>()) as *mut MemBlockFoot
    }
    ///获取下一块的head
    pub fn get_nxt_block(&mut self) -> *mut MemBlockHead {
        (self.addr() + self.size()) as *mut MemBlockHead
    }
    ///获取上一块的head
    pub fn get_pre_block(&mut self) -> *mut MemBlockHead {
        unsafe { (*((self.addr() - size_of::<usize>()) as *mut MemBlockFoot)).get_head() }
    }
}

pub struct MemBlockFoot {
    pub size: usize, //最低位标记是否使用过
}

impl MemBlockFoot {
    ///获取这一块的地址
    pub fn addr(&mut self) -> usize {
        (self as *mut MemBlockFoot as usize) - self.size() + size_of::<usize>()
    }
    //获取这一块的大小
    pub fn size(&self) -> usize {
        self.size >> 1
    }
    ///获取这一块的使用位
    pub fn used(&self) -> bool {
        (self.size & 1) == 1
    }
    ///重设这一块的大小
    pub fn set_size(&mut self, size: usize) {
        self.size = (size << 1) | (self.size & 1);
    }
    ///重设这一块的使用位
    pub fn set_used(&mut self, used: bool) {
        if used {
            self.size |= 1_usize;
        } else {
            self.size &= !1_usize;
        }
    }
    ///获取这一块的head
    pub fn get_head(&mut self) -> *mut MemBlockHead {
        self.addr() as *mut MemBlockHead
    }
}

#[derive(Copy, Clone)]
pub struct LinkedList {
    pub head: *mut MemBlockHead,
}

unsafe impl Send for LinkedList {}

impl LinkedList {
    /// Create a new LinkedList
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
        }
    }

    /// Return `true` if the list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// Push `item` to the front of the list
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn push(&mut self, item: *mut MemBlockHead, size: usize) {
        (*item).nxt = self.head;
        (*item).pre = ptr::null_mut();
        (*item).set_size(size);
        (*item).set_used(false);
        if !self.is_empty() {
            (*self.head).pre = item;
        }
        self.head = item;
    }

    /// Try to remove the first item in the list
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn pop(&mut self) -> Option<*mut MemBlockHead> {
        match self.is_empty() {
            true => None,
            false => {
                // Advance head pointer
                let item = self.head;
                self.head = (*item).nxt;
                (*self.head).pre = ptr::null_mut();
                Some(item)
            }
        }
    }

    /// Try to remove an item in the list
    /// ensure this item is in the list now
    /// # Safety
    /// This function is unsafe because it can cause undefined behavior if the
    /// given address is invalid.
    pub unsafe fn del(&mut self, item: *mut MemBlockHead) {
        if self.head == item {
            self.head = (*item).nxt;
        } else {
            (*(*item).pre).nxt = (*item).nxt;
        }
        if !(*item).nxt.is_null() {
            (*(*item).nxt).pre = (*item).pre;
        }
    }
}
