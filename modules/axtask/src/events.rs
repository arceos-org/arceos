use kspin::SpinNoIrq;

use crate::WaitQueue;

pub struct Event {
    state: SpinNoIrq<LockState>,
}

unsafe impl Sync for Event {}

#[derive(Debug)]
enum LockState {
    Unlocked,
    Locked(WaitQueue),
}

impl Default for LockState {
    fn default() -> Self {
        LockState::Locked(WaitQueue::new())
    }
}

impl Event {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: SpinNoIrq::new(LockState::Locked(WaitQueue::new())),
        }
    }

    #[must_use]
    pub const fn new_set() -> Self {
        Self {
            state: SpinNoIrq::new(LockState::Unlocked),
        }
    }

    /// Return wether the [`Event`] is set or not
    pub fn is_set(&self) -> bool {
        match *self.state.lock() {
            LockState::Unlocked => true,
            _ => false,
        }
    }

    pub fn wait(&self) {
        let state = &mut *self.state.lock();
        match state {
            LockState::Unlocked => {}
            &mut LockState::Locked(ref wq) => {
                wq.wait();
            }
        }
    }

    /// Clears the event (non-blocking).
    ///
    /// If the event was set, it will be cleared and the function returns true.
    /// If the event was unset, the function returns false.
    pub fn clear(&self) -> bool {
        let state = &mut *self.state.lock();
        match state {
            LockState::Unlocked => {
                *state = LockState::Locked(WaitQueue::new());
                true
            }
            LockState::Locked(_) => false,
        }
    }

    pub fn set(&self) {
        let state = &mut *self.state.lock();
        match state {
            LockState::Unlocked => {}
            &mut LockState::Locked(ref wq) => {
                wq.notify_all(true);
            }
        }
    }
}

impl Default for Event {
	fn default() -> Self {
		Self::new()
	}
}
