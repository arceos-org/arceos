use core::{marker::PhantomData, ptr::NonNull};
use super::ListNode;
use super::access::ListAccess;

/// 生成一个通过成员反向获取父类的类型
#[macro_export]
macro_rules! inlist_access {
    ($vis: vis $name: ident, $T: ty, $field: ident) => {
        $vis struct $name {}
        impl $crate::list::access::ListAccess<$T, $crate::list::instrusive::InListNode<$T, Self>>
            for $name
        {
            #[inline(always)]
            fn offset() -> usize {
                $crate::offset_of!($T, $field)
            }
        }
    };
}

/// 侵入式链表头节点
///
/// 如果A为ListAccess<T, Self>则可以访问对应位置
#[repr(transparent)]
pub struct InListNode<T, A = ()> {
    node: ListNode<PhantomData<(T, A)>>,
}

impl<T, A> InListNode<T, A> {
    pub const fn new() -> Self {
        Self {
            node: ListNode::new(PhantomData),
        }
    }
    /// Safety
    /// It must be called after moved.
    pub fn init(&mut self) {
        self.node.init()
    }
    pub fn lazy_init(&mut self) {
        self.node.lazy_init()
    }
    pub fn list_check(&self) {
        self.node.list_check();
    }
    pub fn is_empty(&self) -> bool {
        self.node.is_empty()
    }
    pub fn get_prev(&self) -> *mut Self {
        self.node.get_prev() as *mut _
    }
    pub fn get_next(&self) -> *mut Self {
        self.node.get_next() as *mut _
    }
    pub fn push_prev(&mut self, new: &mut Self) {
        self.node.push_prev(&mut new.node)
    }
    pub fn push_next(&mut self, new: &mut Self) {
        self.node.push_next(&mut new.node)
    }
    pub fn try_prev(&self) -> Option<NonNull<Self>> {
        unsafe { core::mem::transmute(self.node.try_prev()) }
    }
    pub fn try_next(&self) -> Option<NonNull<Self>> {
        unsafe { core::mem::transmute(self.node.try_next()) }
    }
    pub fn pop_self(&mut self) {
        self.node.pop_self()
    }
    pub fn pop_prev(&mut self) -> Option<NonNull<Self>> {
        unsafe { core::mem::transmute(self.node.pop_prev()) }
    }
    pub fn pop_next(&mut self) -> Option<NonNull<Self>> {
        unsafe { core::mem::transmute(self.node.pop_next()) }
    }
    /// 整个链表上只有两个节点
    pub fn is_last(&self) -> bool {
        debug_assert!(!self.is_empty());
        self.node.get_prev() == self.node.get_next()
    }
    /// 不包括头节点的节点数
    pub fn len(&self) -> usize {
        let mut n = 0;
        let mut cur = self.node.get_next();
        let end = &self.node as *const _ as *mut _;
        while cur != end {
            n += 1;
            cur = unsafe { (*cur).next };
        }
        n
    }
}

impl<T, A: ListAccess<T, Self>> InListNode<T, A> {
    /// # Safety
    /// 
    /// 用户自行保证使用的安全性, 下面是获取多个mut的反例
    ///
    /// &mut A(1, 2) -> (&mut A.1, &A.2)
    ///
    /// &A.2 -> &A(1, 2) -> &A.1
    ///
    /// now we hold &mut A.1 and &A.1 at the same time.
    pub unsafe fn access(&self) -> &T {
        A::get(self)
    }
    /// # Safety
    /// 
    /// 用户自行保证唯一性, 下面是获取多个mut的反例
    ///
    /// &mut A(1, 2) -> (&mut A.1, &mut A.2)
    ///
    /// &mut A.2 -> &mut A(1, 2) -> &mut A.1
    ///
    /// now we hold two &mut A.1 at the same time.
    pub unsafe fn access_mut(&mut self) -> &mut T {
        A::get_mut(self)
    }
}

impl<T: 'static, A: ListAccess<T, Self>> InListNode<T, A> {
    pub fn next_iter(&self) -> impl Iterator<Item = &'static T> {
        struct Iter<T, A> {
            cur: *const ListNode<PhantomData<(T, A)>>,
            end: *const ListNode<PhantomData<(T, A)>>,
            offset: usize,
        }

        impl<T: 'static, A> Iterator for Iter<T, A> {
            type Item = &'static T;
            fn next(&mut self) -> Option<Self::Item> {
                if self.cur == self.end {
                    return None;
                }
                let ret = self.cur;
                unsafe {
                    self.cur = (*self.cur).next;
                    Some(&*ret.cast::<u8>().sub(self.offset).cast())
                }
            }
        }

        Iter {
            cur: (self.node.next) as *const _,
            end: core::ptr::addr_of!(self.node),
            offset: A::offset(),
        }
    }
    pub fn next_iter_mut(&mut self) -> impl Iterator<Item = &'static mut T> {
        struct Iter<T, A> {
            cur: *mut ListNode<PhantomData<(T, A)>>,
            end: *mut ListNode<PhantomData<(T, A)>>,
            offset: usize,
        }

        impl<T: 'static, A> Iterator for Iter<T, A> {
            type Item = &'static mut T;
            fn next(&mut self) -> Option<Self::Item> {
                if self.cur == self.end {
                    return None;
                }
                let ret = self.cur;
                unsafe {
                    self.cur = (*self.cur).next;
                    Some(&mut *ret.cast::<u8>().sub(self.offset).cast())
                }
            }
        }

        Iter {
            cur: self.node.next,
            end: core::ptr::addr_of_mut!(self.node),
            offset: A::offset(),
        }
    }
}
