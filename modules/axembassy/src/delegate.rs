use core::{
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicU8, Ordering},
};

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

/// A cell that can only be used on the executor that created it
pub struct SameExecutorCell<T> {
    /// The executor id
    id: usize,
    inner: T,
}

impl<T> SameExecutorCell<T> {
    /// Creates a new `SameExecutorCell` with the provided `inner` value and associates it
    /// with the executor identified by the given `spawner`.
    ///
    /// # Arguments
    ///
    /// * `inner` - The value to be stored inside the cell.
    /// * `spawner` - The spawner used to identify the executor, from which the cell can be accessed.
    pub fn new(inner: T, spawner: Spawner) -> Self {
        Self {
            id: spawner.executor_id(),
            inner,
        }
    }

    /// Creates a new `SameExecutorCell` with the provided `inner` value and associates it with the current executor.
    pub async fn new_async(inner: T) -> Self {
        let spawner = Spawner::for_current_executor().await;
        SameExecutorCell::new(inner, spawner)
    }

    /// Returns a reference to the inner value if the `spawner` matches the executor id associated with `self`.
    pub fn get(&self, spawner: Spawner) -> Option<&T> {
        if self.id == spawner.executor_id() {
            Some(&self.inner)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner value if the `spawner` matches the executor id
    /// associated with `self`.
    pub fn get_mut(&mut self, spawner: Spawner) -> Option<&mut T> {
        if self.id == spawner.executor_id() {
            Some(&mut self.inner)
        } else {
            None
        }
    }

    /// Returns a reference to the inner value by the spawner of current async closure
    pub async fn get_async(&self) -> Option<&T> {
        let spawner = Spawner::for_current_executor().await;
        self.get(spawner)
    }

    /// Returns a mutable reference to the inner value by the spawner of current async closure
    pub async fn get_mut_async(&mut self) -> Option<&mut T> {
        let spawner = Spawner::for_current_executor().await;
        self.get_mut(spawner)
    }

    /// Consumes the `SameExecutorCell`, returning the inner value if the provided `spawner`
    /// matches the executor id associated with `self`.
    ///
    /// else returns `self` as recovery
    pub fn into_inner(self, spawner: Spawner) -> Result<T, Self> {
        if spawner.executor_id() == self.id {
            Ok(self.inner)
        } else {
            Err(self)
        }
    }

    /// Consumes the `SameExecutorCell`, returning the inner value by the spawner of current async closure
    ///
    /// else returns `self` as recovery
    pub async fn into_inner_async(self) -> Result<T, Self> {
        let spawner = Spawner::for_current_executor().await;
        self.into_inner(spawner)
    }

    /// Returns the executor id
    pub fn executor_id(&self) -> usize {
        self.id
    }
}

impl<T: Clone> Clone for SameExecutorCell<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for SameExecutorCell<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SameExecutorCell")
            .field("executor_id", &self.id)
            .field("inner", &self.inner) // Only if T: Debug
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelegateError {
    LendInvalid,
    WithInvalid,
    ConsumedInvalid,
}

#[repr(u8)]
pub enum DelegateState {
    New = 0,
    Lent = 1,
    Consumed = 2,
}

impl From<u8> for DelegateState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::New,
            1 => Self::Lent,
            2 => Self::Consumed,
            _ => unreachable!(),
        }
    }
}

type MutexSignal<T> = Signal<CriticalSectionRawMutex, T>;

pub struct Delegate<T> {
    send: MutexSignal<SameExecutorCell<*mut T>>,
    reply: MutexSignal<()>,
    state: AtomicU8,
    _not_send: PhantomData<*const ()>,
}

unsafe impl<T> Sync for Delegate<T> {}

impl<T> Delegate<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            send: Signal::new(),
            reply: Signal::new(),
            state: AtomicU8::new(DelegateState::New as u8),
            _not_send: PhantomData,
        }
    }

    /// lend
    pub async fn lend<'a, 'b: 'a>(&'a self, target: &'b mut T) -> Result<(), DelegateError> {
        use DelegateError::*;
        use DelegateState::*;

        match self.state.compare_exchange(
            New as u8,
            Lent as u8,
            core::sync::atomic::Ordering::AcqRel,
            core::sync::atomic::Ordering::Acquire,
        ) {
            Ok(_) => {}
            Err(_) => return Err(LendInvalid),
        }
        let sp = Spawner::for_current_executor().await;
        let ptr = ptr::from_mut(target);
        self.send.signal(SameExecutorCell::new(ptr, sp));

        self.reply.wait().await;
        let final_state = self.state.load(Ordering::Acquire);
        if final_state != Consumed as u8 {
            return Err(ConsumedInvalid);
        }
        Ok(())
    }

    /// lend and reset
    pub async fn lend_new<'a, 'b: 'a>(&'a self, target: &'b mut T) -> Result<(), DelegateError> {
        match self.lend(target).await {
            Ok(()) => {
                self.reset();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn with<U>(&self, func: impl FnOnce(&mut T) -> U) -> Result<U, DelegateError> {
        use DelegateError::*;
        use DelegateState::*;

        let data = self.send.wait().await;
        let sp = Spawner::for_current_executor().await;
        let res = {
            let ptr = unsafe { data.get(sp).unwrap().as_mut().unwrap() };
            func(ptr)
        };

        match self.state.compare_exchange(
            Lent as u8,
            Consumed as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {}
            Err(_) => return Err(WithInvalid),
        }

        self.reply.signal(());
        Ok(res)
    }

    pub fn reset(&self) {
        use DelegateState::*;

        let cur_state = self.state.load(core::sync::atomic::Ordering::Acquire);
        if cur_state == Lent as u8 {
            panic!(
                "Cannot reset Delegate while in LENT state: lend() called but with() has not completed."
            );
        }

        // Case:
        // 1. New: Refresh nothing.
        // 2. Consumed: Refresh Send, Reset is `()`, refresh nothing.
        if cur_state == New as u8 {
            return;
        }

        self.send.reset();
        self.state.store(New as u8, Ordering::Release);
    }
}

impl<T> Default for Delegate<T> {
    fn default() -> Self {
        Self::new()
    }
}
