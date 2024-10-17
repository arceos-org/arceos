use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

#[cfg(feature = "multitask")]
mod multitask;
#[cfg(feature = "multitask")]
use multitask as sys;

#[cfg(not(feature = "multitask"))]
mod no_thread;
#[cfg(not(feature = "multitask"))]
use no_thread as sys;

/// A reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// In comparison, a [`Mutex`] does not distinguish between readers or writers
/// that acquire the lock, therefore blocking any threads waiting for the lock to
/// become available. An `RwLock` will allow any number of readers to acquire the
/// lock as long as a writer is not holding the lock.
///
/// The priority policy of the lock is dependent on the underlying operating
/// system's implementation, and this type does not guarantee that any
/// particular policy will be used. In particular, a writer which is waiting to
/// acquire the lock in `write` might or might not block concurrent calls to
/// `read`, e.g.:
///
/// <details><summary>Potential deadlock example</summary>
///
/// ```text
/// // Thread 1              |  // Thread 2
/// let _rg1 = lock.read();  |
///                          |  // will block
///                          |  let _wg = lock.write();
/// // may deadlock          |
/// let _rg2 = lock.read();  |
/// ```
///
/// </details>
///
/// The type parameter `T` represents the data that this lock protects. It is
/// required that `T` satisfies [`Send`] to be shared across threads and
/// [`Sync`] to allow concurrent access through readers. The RAII guards
/// returned from the locking methods implement [`Deref`] (and [`DerefMut`]
/// for the `write` methods) to allow access to the content of the lock.
///
/// [`Mutex`]: super::Mutex
pub struct RwLock<T: ?Sized> {
    inner: sys::RwLock,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}

unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] and [`try_read`] methods on
/// [`RwLock`].
///
/// [`read`]: RwLock::read
/// [`try_read`]: RwLock::try_read
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct RwLockReadGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a T` to avoid `noalias` violations, because a
    // `RwLockReadGuard` argument doesn't hold immutability for its whole scope, only until it drops.
    // `NonNull` is also covariant over `T`, just like we would have with `&T`. `NonNull`
    // is preferable over `const* T` to allow for niche optimization.
    data: NonNull<T>,
    inner_lock: &'a sys::RwLock,
}

// impl<T: ?Sized> !Send for RwLockReadGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for RwLockReadGuard<'_, T> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and [`try_write`] methods
/// on [`RwLock`].
///
/// [`write`]: RwLock::write
/// [`try_write`]: RwLock::try_write
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
}

// impl<T: ?Sized> !Send for RwLockWriteGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}

/// RAII structure used to release the shared read access of a lock when
/// dropped, which can point to a subfield of the protected data.
///
/// This structure is created by the [`map`] and [`try_map`] methods
/// on [`RwLockReadGuard`].
///
/// [`map`]: RwLockReadGuard::map
/// [`try_map`]: RwLockReadGuard::try_map
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct MappedRwLockReadGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a T` to avoid `noalias` violations, because a
    // `MappedRwLockReadGuard` argument doesn't hold immutability for its whole scope, only until it drops.
    // `NonNull` is also covariant over `T`, just like we would have with `&T`. `NonNull`
    // is preferable over `const* T` to allow for niche optimization.
    data: NonNull<T>,
    inner_lock: &'a sys::RwLock,
}

// impl<T: ?Sized> !Send for MappedRwLockReadGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockReadGuard<'_, T> {}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped, which can point to a subfield of the protected data.
///
/// This structure is created by the [`map`] and [`try_map`] methods
/// on [`RwLockWriteGuard`].
///
/// [`map`]: RwLockWriteGuard::map
/// [`try_map`]: RwLockWriteGuard::try_map
#[must_use = "if unused the RwLock will immediately unlock"]
#[clippy::has_significant_drop]
pub struct MappedRwLockWriteGuard<'a, T: ?Sized + 'a> {
    // NB: we use a pointer instead of `&'a mut T` to avoid `noalias` violations, because a
    // `MappedRwLockWriteGuard` argument doesn't hold uniqueness for its whole scope, only until it drops.
    // `NonNull` is covariant over `T`, so we add a `PhantomData<&'a mut T>` field
    // below for the correct variance over `T` (invariance).
    data: NonNull<T>,
    inner_lock: &'a sys::RwLock,
    _variance: PhantomData<&'a mut T>,
}

// impl<T: ?Sized> !Send for MappedRwLockWriteGuard<'_, T> {}

unsafe impl<T: ?Sized + Sync> Sync for MappedRwLockWriteGuard<'_, T> {}

impl<T> RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    #[inline]
    pub const fn new(t: T) -> RwLock<T> {
        RwLock {
            inner: sys::RwLock::new(),
            data: UnsafeCell::new(t),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Locks this `RwLock` with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns. This method does not provide any guarantees with
    /// respect to the ordering of whether contentious readers or writers will
    /// acquire the lock first.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        unsafe {
            self.inner.read();
            RwLockReadGuard::new(self)
        }
    }

    /// Attempts to acquire this `RwLock` with shared read access.
    ///
    /// If the access could not be granted at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned which will release the shared access
    /// when it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    ///
    #[inline]
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        unsafe {
            if self.inner.try_read() {
                Some(RwLockReadGuard::new(self))
            } else {
                None
            }
        }
    }

    /// Locks this `RwLock` with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this `RwLock`
    /// when dropped.
    ///
    /// # Panics
    ///
    /// This function might panic when called if the lock is already held by the current thread.
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        unsafe {
            self.inner.write();
            RwLockWriteGuard::new(self)
        }
    }

    /// Attempts to lock this `RwLock` with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// it is dropped.
    ///
    /// This function does not block.
    ///
    /// This function does not provide any guarantees with respect to the ordering
    /// of whether contentious readers or writers will acquire the lock first.
    #[inline]

    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        unsafe {
            if self.inner.try_write() {
                Some(RwLockWriteGuard::new(self))
            } else {
                None
            }
        }
    }

    /// Consumes this `RwLock`, returning the underlying data.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `RwLock` is poisoned. An
    /// `RwLock` is poisoned whenever a writer panics while holding an exclusive
    /// lock. An error will only be returned if the lock would have otherwise
    /// been acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::RwLock;
    ///
    /// let lock = RwLock::new(String::new());
    /// {
    ///     let mut s = lock.write().unwrap();
    ///     *s = "modified".to_owned();
    /// }
    /// assert_eq!(lock.into_inner().unwrap(), "modified");
    /// ```
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `RwLock` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("RwLock");
        match self.try_read() {
            Some(guard) => {
                d.field("data", &&*guard);
            }
            None => {
                d.field("data", &format_args!("<locked>"));
            }
        }
        d.finish_non_exhaustive()
    }
}

impl<T: Default> Default for RwLock<T> {
    /// Creates a new `RwLock<T>`, with the `Default` value for T.
    fn default() -> RwLock<T> {
        RwLock::new(Default::default())
    }
}

impl<T> From<T> for RwLock<T> {
    /// Creates a new instance of an `RwLock<T>` which is unlocked.
    /// This is equivalent to [`RwLock::new`].
    fn from(t: T) -> Self {
        RwLock::new(t)
    }
}

impl<'rwlock, T: ?Sized> RwLockReadGuard<'rwlock, T> {
    /// Creates a new instance of `RwLockReadGuard<T>` from a `RwLock<T>`.
    // SAFETY: if and only if `lock.inner.read()` (or `lock.inner.try_read()`) has been
    // successfully called from the same thread before instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T>) -> RwLockReadGuard<'rwlock, T> {
        RwLockReadGuard {
            data: unsafe { NonNull::new_unchecked(lock.data.get()) },
            inner_lock: &lock.inner,
        }
    }
}

impl<'rwlock, T: ?Sized> RwLockWriteGuard<'rwlock, T> {
    /// Creates a new instance of `RwLockWriteGuard<T>` from a `RwLock<T>`.
    // SAFETY: if and only if `lock.inner.write()` (or `lock.inner.try_write()`) has been
    // successfully called from the same thread before instantiating this object.
    unsafe fn new(lock: &'rwlock RwLock<T>) -> RwLockWriteGuard<'rwlock, T> {
        RwLockWriteGuard { lock }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MappedRwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for MappedRwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MappedRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for MappedRwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Deref for MappedRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> Deref for MappedRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe { self.data.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for MappedRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe { self.data.as_mut() }
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when created.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when created.
        unsafe {
            self.lock.inner.write_unlock();
        }
    }
}

impl<T: ?Sized> Drop for MappedRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe {
            self.inner_lock.read_unlock();
        }
    }
}

impl<T: ?Sized> Drop for MappedRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        unsafe {
            self.inner_lock.write_unlock();
        }
    }
}

impl<'a, T: ?Sized> RwLockReadGuard<'a, T> {
    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::map(...)`. A method would interfere with methods of
    /// the same name on the contents of the `RwLockReadGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be poisoned.
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_ref() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockReadGuard {
            data,
            inner_lock: &orig.inner_lock,
        }
    }

    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::try_map(...)`. A method would interfere with methods
    /// of the same name on the contents of the `RwLockReadGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedRwLockReadGuard<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_ref() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockReadGuard {
                    data,
                    inner_lock: &orig.inner_lock,
                })
            }
            None => Err(orig),
        }
    }
}

impl<'a, T: ?Sized> MappedRwLockReadGuard<'a, T> {
    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data,
    /// e.g. an enum variant.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockReadGuard::map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockReadGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be poisoned.
    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_ref() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockReadGuard {
            data,
            inner_lock: &orig.inner_lock,
        }
    }

    /// Makes a [`MappedRwLockReadGuard`] for a component of the borrowed data.
    /// The original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for reading, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockReadGuard::try_map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockReadGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will not be poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedRwLockReadGuard<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockReadGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_ref() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockReadGuard {
                    data,
                    inner_lock: &orig.inner_lock,
                })
            }
            None => Err(orig),
        }
    }
}

impl<'a, T: ?Sized> RwLockWriteGuard<'a, T> {
    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data, e.g.
    /// an enum variant.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockWriteGuard::map(...)`. A method would interfere with methods of
    /// the same name on the contents of the `RwLockWriteGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.

    pub fn map<U, F>(orig: Self, f: F) -> MappedRwLockWriteGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { &mut *orig.lock.data.get() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockWriteGuard {
            data,
            inner_lock: &orig.lock.inner,
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data. The
    /// original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockWriteGuard::try_map(...)`. A method would interfere with methods
    /// of the same name on the contents of the `RwLockWriteGuard` used through
    /// `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(orig: Self, f: F) -> Result<MappedRwLockWriteGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { &mut *orig.lock.data.get() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockWriteGuard {
                    data,
                    inner_lock: &orig.lock.inner,
                    _variance: PhantomData,
                })
            }
            None => Err(orig),
        }
    }
}

impl<'a, T: ?Sized> MappedRwLockWriteGuard<'a, T> {
    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data,
    /// e.g. an enum variant.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockWriteGuard::map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockWriteGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    pub fn map<U, F>(mut orig: Self, f: F) -> MappedRwLockWriteGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        let data = NonNull::from(f(unsafe { orig.data.as_mut() }));
        let orig = ManuallyDrop::new(orig);
        MappedRwLockWriteGuard {
            data,
            inner_lock: orig.inner_lock,
            _variance: PhantomData,
        }
    }

    /// Makes a [`MappedRwLockWriteGuard`] for a component of the borrowed data.
    /// The original guard is returned as an `Err(...)` if the closure returns
    /// `None`.
    ///
    /// The `RwLock` is already locked for writing, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `MappedRwLockWriteGuard::try_map(...)`. A method would interfere with
    /// methods of the same name on the contents of the `MappedRwLockWriteGuard`
    /// used through `Deref`.
    ///
    /// # Panics
    ///
    /// If the closure panics, the guard will be dropped (unlocked) and the RwLock will be poisoned.
    #[doc(alias = "filter_map")]
    pub fn try_map<U, F>(mut orig: Self, f: F) -> Result<MappedRwLockWriteGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
        U: ?Sized,
    {
        // SAFETY: the conditions of `RwLockWriteGuard::new` were satisfied when the original guard
        // was created, and have been upheld throughout `map` and/or `try_map`.
        // The signature of the closure guarantees that it will not "leak" the lifetime of the reference
        // passed to it. If the closure panics, the guard will be dropped.
        match f(unsafe { orig.data.as_mut() }) {
            Some(data) => {
                let data = NonNull::from(data);
                let orig = ManuallyDrop::new(orig);
                Ok(MappedRwLockWriteGuard {
                    data,
                    inner_lock: orig.inner_lock,
                    _variance: PhantomData,
                })
            }
            None => Err(orig),
        }
    }
}
