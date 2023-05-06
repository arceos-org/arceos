// SPDX-License-Identifier: GPL-2.0

//! Linked lists.
//!
//! Based on linux/rust/kernel/linked_list.rs, but use
//! [`unsafe_list::List`] as the inner implementation.
//!
//! TODO: This module is a work in progress.

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::ptr::NonNull;

use crate::unsafe_list::{self, Adapter, Cursor, Links};

// TODO: Use the one from `kernel::file_operations::PointerWrapper` instead.
/// Wraps an object to be inserted in a linked list.
pub trait Wrapper<T: ?Sized> {
    /// Converts the wrapped object into a pointer that represents it.
    fn into_pointer(self) -> NonNull<T>;

    /// Converts the object back from the pointer representation.
    ///
    /// # Safety
    ///
    /// The passed pointer must come from a previous call to [`Wrapper::into_pointer()`].
    unsafe fn from_pointer(ptr: NonNull<T>) -> Self;

    /// Returns a reference to the wrapped object.
    fn as_ref(&self) -> &T;
}

impl<T: ?Sized> Wrapper<T> for Box<T> {
    #[inline]
    fn into_pointer(self) -> NonNull<T> {
        NonNull::new(Box::into_raw(self)).unwrap()
    }

    #[inline]
    unsafe fn from_pointer(ptr: NonNull<T>) -> Self {
        unsafe { Box::from_raw(ptr.as_ptr()) }
    }

    #[inline]
    fn as_ref(&self) -> &T {
        AsRef::as_ref(self)
    }
}

impl<T: ?Sized> Wrapper<T> for Arc<T> {
    #[inline]
    fn into_pointer(self) -> NonNull<T> {
        NonNull::new(Arc::into_raw(self) as _).unwrap()
    }

    #[inline]
    unsafe fn from_pointer(ptr: NonNull<T>) -> Self {
        // SAFETY: The safety requirements of `from_pointer` satisfy the ones from `Arc::from_raw`.
        unsafe { Arc::from_raw(ptr.as_ptr() as _) }
    }

    #[inline]
    fn as_ref(&self) -> &T {
        AsRef::as_ref(self)
    }
}

impl<T: ?Sized> Wrapper<T> for &T {
    #[inline]
    fn into_pointer(self) -> NonNull<T> {
        NonNull::from(self)
    }

    #[inline]
    unsafe fn from_pointer(ptr: NonNull<T>) -> Self {
        unsafe { &*ptr.as_ptr() }
    }

    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

/// A descriptor of wrapped list elements.
pub trait AdapterWrapped: Adapter {
    /// Specifies which wrapper (e.g., `Box` and `Arc`) wraps the list entries.
    type Wrapped: Wrapper<Self::EntryType>;
}

impl<T: ?Sized> AdapterWrapped for Box<T>
where
    Box<T>: Adapter,
{
    type Wrapped = Box<<Box<T> as Adapter>::EntryType>;
}

unsafe impl<T: Adapter + ?Sized> Adapter for Box<T> {
    type EntryType = T::EntryType;

    #[inline]
    fn to_links(data: &Self::EntryType) -> &Links<Self::EntryType> {
        <T as Adapter>::to_links(data)
    }
}

impl<T: ?Sized> AdapterWrapped for Arc<T>
where
    Arc<T>: Adapter,
{
    type Wrapped = Arc<<Arc<T> as Adapter>::EntryType>;
}

unsafe impl<T: Adapter + ?Sized> Adapter for Arc<T> {
    type EntryType = T::EntryType;

    #[inline]
    fn to_links(data: &Self::EntryType) -> &Links<Self::EntryType> {
        <T as Adapter>::to_links(data)
    }
}

/// A linked list.
///
/// Elements in the list are wrapped and ownership is transferred to the list while the element is
/// in the list.
pub struct List<G: AdapterWrapped> {
    list: unsafe_list::List<G>,
}

impl<G: AdapterWrapped> List<G> {
    /// Constructs a new empty linked list.
    pub const fn new() -> Self {
        Self {
            list: unsafe_list::List::new(),
        }
    }

    /// Returns whether the list is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Adds the given object to the end (back) of the list.
    ///
    /// It is dropped if it's already on this (or another) list; this can happen for
    /// reference-counted objects, so dropping means decrementing the reference count.
    pub fn push_back(&mut self, data: G::Wrapped) {
        let ptr = data.into_pointer();

        // SAFETY: We took ownership of the entry, so it is safe to insert it.
        unsafe { self.list.push_back(ptr.as_ref()) }
    }

    /// Inserts the given object after `existing`.
    ///
    /// It is dropped if it's already on this (or another) list; this can happen for
    /// reference-counted objects, so dropping means decrementing the reference count.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `existing` points to a valid entry that is on the list.
    pub unsafe fn insert_after(&mut self, existing: NonNull<G::EntryType>, data: G::Wrapped) {
        let ptr = data.into_pointer();
        unsafe { self.list.insert_after(existing, ptr.as_ref()) }
    }

    /// Removes the given entry.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `data` is either on this list. It being on another
    /// list leads to memory unsafety.
    pub unsafe fn remove(&mut self, data: &G::Wrapped) -> Option<G::Wrapped> {
        let entry_ref = Wrapper::as_ref(data);
        unsafe { self.list.remove(entry_ref) };
        Some(unsafe { G::Wrapped::from_pointer(NonNull::from(entry_ref)) })
    }

    /// Removes the element currently at the front of the list and returns it.
    ///
    /// Returns `None` if the list is empty.
    pub fn pop_front(&mut self) -> Option<G::Wrapped> {
        let entry_ref = unsafe { self.list.front()?.as_ref() };
        unsafe { self.list.remove(entry_ref) };
        Some(unsafe { G::Wrapped::from_pointer(NonNull::from(entry_ref)) })
    }

    /// Returns the first element of the list, if one exists.
    #[inline]
    pub fn front(&self) -> Option<&G::EntryType> {
        self.list.front().map(|ptr| unsafe { ptr.as_ref() })
    }

    /// Returns the last element of the list, if one exists.
    #[inline]
    pub fn back(&self) -> Option<&G::EntryType> {
        self.list.back().map(|ptr| unsafe { ptr.as_ref() })
    }

    /// Returns a cursor starting on the first (front) element of the list.
    #[inline]
    pub fn cursor_front(&self) -> Cursor<'_, G> {
        self.list.cursor_front()
    }
}

impl<G: AdapterWrapped> Default for List<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: AdapterWrapped> Drop for List<G> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}
