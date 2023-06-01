#![allow(unused)]
use core::ptr::NonNull;

/// access
pub mod access;
///instrusive
pub mod instrusive;

pub use instrusive::InListNode;

/// Linked list node
pub struct ListNode<T> {
    prev: *mut ListNode<T>,
    next: *mut ListNode<T>,
    data: T,
}

unsafe impl<T> Send for ListNode<T> {}
unsafe impl<T> Sync for ListNode<T> {}

impl<T> ListNode<T> {
    /// Create a new node
    pub const fn new(data: T) -> Self {
        Self {
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
            data,
        }
    }

    /// Init
    pub fn init(&mut self) {
        self.prev = self;
        self.next = self;
    }

    /// Is inited
    pub fn inited(&self) -> bool {
        !self.prev.is_null()
    }

    /// Lazy init
    pub fn lazy_init(&mut self) {
        if !self.inited() {
            debug_assert!(self.next.is_null());
            self.init();
        }
        debug_assert!(!self.next.is_null());
    }

    /// Check valid
    pub fn list_check(&self) {
        if cfg!(debug_assertions) {
            unsafe {
                debug_assert!(!self.prev.is_null());
                debug_assert!(!self.next.is_null());
                let mut cur = self as *const _ as *mut Self;
                let mut nxt = (*cur).next;
                assert!((*nxt).prev == cur);
                cur = nxt;
                nxt = (*cur).next;
                while cur as usize != self as *const Self as usize {
                    assert!((*nxt).prev == cur);
                    cur = nxt;
                    nxt = (*cur).next;
                }
            }
        }
    }

    /// Get data
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get data mut
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Is the linked list empty
    pub fn is_empty(&self) -> bool {
        debug_assert!(self.inited());
        if self.prev as *const _ == self {
            debug_assert!(self.next as *const _ == self);
            true
        } else {
            debug_assert!(self.next as *const _ != self);
            false
        }
    }

    /// # Safety
    ///
    /// 自行保证指针的安全
    pub unsafe fn set_prev(&mut self, prev: *mut Self) {
        self.prev = prev;
    }
    /// # Safety
    ///
    /// 自行保证指针的安全
    pub unsafe fn set_next(&mut self, next: *mut Self) {
        self.next = next;
    }
    /// Get prev
    pub fn get_prev(&self) -> *mut Self {
        self.prev
    }
    /// Get next
    pub fn get_next(&self) -> *mut Self {
        self.next
    }

    /// Push to prev
    pub fn push_prev(&mut self, new: &mut Self) {
        debug_assert!(self as *mut _ != new as *mut _);
        debug_assert!(new.is_empty());
        new.prev = self.prev;
        new.next = self;
        debug_assert!(unsafe { (*self.prev).next == self });
        unsafe { (*self.prev).next = new };
        self.prev = new;
    }

    /// Push to next
    pub fn push_next(&mut self, new: &mut Self) {
        debug_assert!(self as *mut _ != new as *mut _);
        debug_assert!(new.is_empty());
        new.prev = self;
        new.next = self.next;
        debug_assert!(unsafe { (*self.next).prev == self });
        debug_assert!(unsafe { (*self.prev).next == self });
        unsafe { (*self.next).prev = new };
        self.next = new;
    }

    /// Try to get prev
    pub fn try_prev(&self) -> Option<NonNull<Self>> {
        if self.is_empty() {
            return None;
        }
        NonNull::new(self.prev)
    }

    /// Try to get next
    pub fn try_next(&self) -> Option<NonNull<Self>> {
        if self.is_empty() {
            return None;
        }
        NonNull::new(self.next)
    }

    /// Pop itself
    pub fn pop_self(&mut self) {
        debug_assert!(unsafe { (*self.next).prev == self });
        debug_assert!(unsafe { (*self.prev).next == self });
        let prev = self.prev;
        let next = self.next;
        unsafe {
            (*prev).next = next;
            (*next).prev = prev;
        }
        self.init();
    }

    /// Pop prev
    pub fn pop_prev(&mut self) -> Option<NonNull<Self>> {
        if self.is_empty() {
            return None;
        }
        let r = self.prev;
        unsafe {
            debug_assert!((*r).next == self);
            let r_prev = (*r).prev;
            debug_assert!((*r_prev).next == r);
            self.prev = r_prev;
            (*r_prev).next = self;
            (*r).init();
        }
        NonNull::new(r)
    }

    /// Pop next
    pub fn pop_next(&mut self) -> Option<NonNull<Self>> {
        if self.is_empty() {
            return None;
        }
        let r = self.next;
        unsafe {
            debug_assert!((*r).prev == self);
            let r_next = (*r).next;
            debug_assert!((*r_next).prev == r);
            self.next = r_next;
            (*r_next).prev = self;
            (*r).init();
        }
        NonNull::new(r)
    }
}
