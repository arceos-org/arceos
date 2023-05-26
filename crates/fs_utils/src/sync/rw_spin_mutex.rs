#![allow(dead_code)]

use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{self, AtomicIsize, Ordering},
};

use super::MutexSupport;

/// 读优先自旋锁
///
/// 锁状态: 0 -> unlock >0 -> shared lock <0 -> unique lock
///
/// 读者在未获取锁时
///
pub struct RwSpinMutex<T: ?Sized, S: MutexSupport> {
    lock: AtomicIsize,
    _marker: PhantomData<S>,
    data: UnsafeCell<T>, // actual data
}

unsafe impl<T: ?Sized + Send, S: MutexSupport> Send for RwSpinMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexSupport> Sync for RwSpinMutex<T, S> {}

impl<T, S: MutexSupport> RwSpinMutex<T, S> {
    #[inline(always)]
    pub const fn new(user_data: T) -> Self {
        RwSpinMutex {
            lock: AtomicIsize::new(0),
            data: UnsafeCell::new(user_data),
            _marker: PhantomData,
        }
    }

    /// Consumes this mutex, returning the underlying data.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let RwSpinMutex { data, .. } = self;
        data.into_inner()
    }
}

impl<T: ?Sized, S: MutexSupport> RwSpinMutex<T, S> {
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
    /// # Safety
    ///
    /// 自行保证数据访问的安全
    #[inline(always)]
    pub unsafe fn unsafe_get(&self) -> &T {
        &*self.data.get()
    }
    /// # Safety
    ///
    /// 自行保证数据访问的安全
    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub unsafe fn unsafe_get_mut(&self) -> &mut T {
        &mut *self.data.get()
    }
    #[inline(always)]
    pub fn try_unique_lock(&self) -> Option<impl DerefMut<Target = T> + '_> {
        let mut guard = S::before_lock();
        if self.lock.load(Ordering::Relaxed) != 0 {
            return None;
        }
        match self
            .lock
            .compare_exchange(0, -isize::MAX, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => Some(UniqueRwMutexGuard {
                _not_send_sync: PhantomData,
                mutex: self,
                guard,
            }),
            Err(_) => {
                S::after_unlock(&mut guard);
                None
            }
        }
    }
    #[inline(always)]
    pub fn unique_lock(&self) -> impl DerefMut<Target = T> + '_ {
        let guard = S::before_lock();
        loop {
            let mut cnt = 0;
            while self.lock.load(Ordering::Relaxed) != 0 {
                core::hint::spin_loop();
                cnt += 1;
                if cnt == 0x10000000 {
                    panic!("dead lock");
                }
            }
            if self
                .lock
                .compare_exchange(0, -isize::MAX, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                continue;
            }
            return UniqueRwMutexGuard {
                _not_send_sync: PhantomData,
                mutex: self,
                guard,
            };
        }
    }
    #[inline(always)]
    pub fn try_shared_lock(&self) -> Option<impl Deref<Target = T> + '_> {
        let mut guard = S::before_lock();
        let mut cur = self.lock.load(Ordering::Relaxed);
        while cur >= 0 {
            match self
                .lock
                .compare_exchange(cur, cur + 1, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => {
                    return Some(SharedRwMutexGuard {
                        _not_send_sync: PhantomData,
                        mutex: self,
                        guard,
                    })
                }
                Err(v) => cur = v,
            };
        }
        S::after_unlock(&mut guard);
        None
    }
    #[inline(always)]
    pub fn shared_lock(&self) -> impl Deref<Target = T> + '_ {
        let guard = S::before_lock();
        if self.lock.fetch_add(1, Ordering::Relaxed) >= 0 {
            atomic::fence(Ordering::Acquire);
            return SharedRwMutexGuard {
                _not_send_sync: PhantomData,
                mutex: self,
                guard,
            };
        }
        let mut cnt = 0;
        while self.lock.load(Ordering::Relaxed) <= 0 {
            if cnt == 0x10000000 {
                panic!("dead lock");
            }
            cnt += 1;
            core::hint::spin_loop();
        }
        atomic::fence(Ordering::Acquire);
        SharedRwMutexGuard {
            _not_send_sync: PhantomData,
            mutex: self,
            guard,
        }
    }
}

impl<T: ?Sized + Default, S: MutexSupport> Default for RwSpinMutex<T, S> {
    fn default() -> RwSpinMutex<T, S> {
        RwSpinMutex::new(Default::default())
    }
}

struct UniqueRwMutexGuard<'a, T: ?Sized, S: MutexSupport> {
    _not_send_sync: PhantomData<*const ()>,
    mutex: &'a RwSpinMutex<T, S>,
    guard: S::GuardData,
}

struct SharedRwMutexGuard<'a, T: ?Sized, S: MutexSupport + 'a> {
    _not_send_sync: PhantomData<*const ()>,
    mutex: &'a RwSpinMutex<T, S>,
    guard: S::GuardData,
}

impl<'a, T: ?Sized, S: MutexSupport> Deref for SharedRwMutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}
impl<'a, T: ?Sized, S: MutexSupport> Deref for UniqueRwMutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> DerefMut for UniqueRwMutexGuard<'a, T, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Drop for SharedRwMutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    #[inline(always)]
    fn drop(&mut self) {
        debug_assert!(self.mutex.lock.load(Ordering::Relaxed) > 0);
        self.mutex.lock.fetch_sub(1, Ordering::Release);
        S::after_unlock(&mut self.guard);
    }
}
impl<'a, T: ?Sized, S: MutexSupport> Drop for UniqueRwMutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    #[inline(always)]
    fn drop(&mut self) {
        debug_assert!(self.mutex.lock.load(Ordering::Relaxed) < 0);
        // debug_assert_eq!(self.mutex.lock.load(Ordering::Relaxed), -1);
        // self.mutex.lock.store(0, Ordering::Release);
        self.mutex.lock.fetch_add(isize::MAX, Ordering::Release);
        S::after_unlock(&mut self.guard);
    }
}
