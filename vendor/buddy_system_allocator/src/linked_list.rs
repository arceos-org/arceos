//! Provide the intrusive LinkedList
#![allow(dead_code)]

use core::{fmt, ptr};

/// An intrusive linked list
///
/// A clean room implementation of the one used in CS140e 2018 Winter
///
/// Thanks Sergio Benitez for his excellent work,
/// See [CS140e](https://cs140e.sergio.bz/) for more information
#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut usize,
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
    pub unsafe fn push(&mut self, item: *mut usize) {
        *item = self.head as usize;
        self.head = item;
    }

    /// Try to remove the first item in the list
    pub fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                // Advance head pointer
                let item = self.head;
                self.head = unsafe { *item as *mut usize };
                Some(item)
            }
        }
    }

    /// Return an iterator over the items in the list
    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,
            list: self,
        }
    }

    /// Return an mutable iterator over the items in the list
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            list: self,
        }
    }
}

impl fmt::Debug for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An iterator over the linked list
pub struct Iter<'a> {
    curr: *mut usize,
    list: &'a LinkedList,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let item = self.curr;
            let next = unsafe { *item as *mut usize };
            self.curr = next;
            Some(item)
        }
    }
}

/// Represent a mutable node in `LinkedList`
pub struct ListNode {
    prev: *mut usize,
    curr: *mut usize,
}

impl ListNode {
    /// Remove the node from the list
    pub fn pop(self) -> *mut usize {
        // Skip the current one
        unsafe {
            *(self.prev) = *(self.curr);
        }
        self.curr
    }

    /// Returns the pointed address
    pub fn value(&self) -> *mut usize {
        self.curr
    }
}

/// A mutable iterator over the linked list
pub struct IterMut<'a> {
    list: &'a mut LinkedList,
    prev: *mut usize,
    curr: *mut usize,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let res = ListNode {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe { *self.curr as *mut usize };
            Some(res)
        }
    }
}
