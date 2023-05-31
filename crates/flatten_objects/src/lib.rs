//! [`FlattenObjects`] is a container that stores numbered objects.
//!
//! Objects can be added to the [`FlattenObjects`], a unique ID will be assigned
//! to the object. The ID can be used to retrieve the object later.
//!
//! # Example
//!
//! ```
//! use flatten_objects::FlattenObjects;
//!
//! let mut objects = FlattenObjects::<u32, 20>::new();
//!
//! // Add `23` 10 times and assign them IDs from 0 to 9.
//! for i in 0..=9 {
//!     objects.add_at(i, 23).unwrap();
//!     assert!(objects.is_assigned(i));
//! }
//!
//! // Remove the object with ID 6.
//! assert_eq!(objects.remove(6), Some(23));
//! assert!(!objects.is_assigned(6));
//!
//! // Add `42` (the ID 6 is available now).
//! let id = objects.add(42).unwrap();
//! assert_eq!(id, 6);
//! assert!(objects.is_assigned(id));
//! assert_eq!(objects.get(id), Some(&42));
//! assert_eq!(objects.remove(id), Some(42));
//! assert!(!objects.is_assigned(id));
//! ```

#![no_std]
#![feature(const_maybe_uninit_zeroed)]
#![feature(maybe_uninit_uninit_array)]
#![feature(const_maybe_uninit_uninit_array)]

use bitmaps::Bitmap;
use core::mem::MaybeUninit;

/// A container that stores numbered objects.
///
/// See the [crate-level documentation](crate) for more details.
///
/// `CAP` is the maximum number of objects that can be held. It also equals the
/// maximum ID that can be assigned plus one. Currently, `CAP` must not be
/// greater than 1024.
pub struct FlattenObjects<T, const CAP: usize> {
    objects: [MaybeUninit<T>; CAP],
    id_bitmap: Bitmap<1024>,
    count: usize,
}

impl<T, const CAP: usize> FlattenObjects<T, CAP> {
    /// Creates a new empty `FlattenObjects`.
    pub const fn new() -> Self {
        assert!(CAP <= 1024);
        Self {
            objects: MaybeUninit::uninit_array(),
            // SAFETY: zero initialization is OK for `id_bitmap` (an array of integers).
            id_bitmap: unsafe { MaybeUninit::zeroed().assume_init() },
            count: 0,
        }
    }

    /// Returns the maximum number of objects that can be held.
    ///
    /// It also equals the maximum ID that can be assigned plus one.
    #[inline]
    pub const fn capacity(&self) -> usize {
        CAP
    }

    /// Returns the number of objects that have been added.
    #[inline]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns `true` if the given `id` is already be assigned.
    #[inline]
    pub fn is_assigned(&self, id: usize) -> bool {
        id < CAP && self.id_bitmap.get(id)
    }

    /// Returns the reference of the element with the given `id` if it already
    /// be assigned. Otherwise, returns `None`.
    #[inline]
    pub fn get(&self, id: usize) -> Option<&T> {
        if self.is_assigned(id) {
            // SAFETY: the object at `id` should be initialized by `add` or
            // `add_at`.
            unsafe { Some(self.objects[id].assume_init_ref()) }
        } else {
            None
        }
    }

    /// Returns the mutable reference of the element with the given `id` if it
    /// exists. Otherwise, returns `None`.
    #[inline]
    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        if self.is_assigned(id) {
            // SAFETY: the object at `id` should be initialized by `add` or
            // `add_at`.
            unsafe { Some(self.objects[id].assume_init_mut()) }
        } else {
            None
        }
    }

    /// Add an object and assigns it a unique ID.
    ///
    /// Returns the ID if there is one available. Otherwise, returns `None`.
    pub fn add(&mut self, value: T) -> Option<usize> {
        let id = self.id_bitmap.first_false_index()?;
        if id < CAP {
            self.count += 1;
            self.id_bitmap.set(id, true);
            self.objects[id].write(value);
            Some(id)
        } else {
            None
        }
    }

    /// Add an object and assigns it a specific ID.
    ///
    /// Returns the ID if it's not used by others. Otherwise, returns `None`.
    pub fn add_at(&mut self, id: usize, value: T) -> Option<usize> {
        if self.is_assigned(id) {
            return None;
        }
        self.count += 1;
        self.id_bitmap.set(id, true);
        self.objects[id].write(value);
        Some(id)
    }

    /// Removes the object with the given ID.
    ///
    /// Returns the object if there is one assigned to the ID. Otherwise,
    /// returns `None`.
    ///
    /// After this operation, the ID is freed and can be assigned for next
    /// object again.
    pub fn remove(&mut self, id: usize) -> Option<T> {
        if self.is_assigned(id) {
            self.id_bitmap.set(id, false);
            self.count -= 1;
            // SAFETY: the object at `id` should be initialized by `add` or
            // `add_at`, and can not be retrieved by `get` or `get_mut` unless
            // it be added again.
            unsafe { Some(self.objects[id].assume_init_read()) }
        } else {
            None
        }
    }
}
