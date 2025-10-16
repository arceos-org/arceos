use core::sync::atomic::{AtomicU8, Ordering};

use axerrno::AxResult;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Idle,
    Busy,
    Connecting,
    Connected,
    Listening,
    Closed,
}

impl TryFrom<u8> for State {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        Ok(match value {
            0 => State::Idle,
            1 => State::Busy,
            2 => State::Connecting,
            3 => State::Connected,
            4 => State::Listening,
            5 => State::Closed,
            _ => return Err(()),
        })
    }
}

pub struct StateLock(AtomicU8);
impl StateLock {
    pub fn new(state: State) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    pub fn get(&self) -> State {
        self.0
            .load(Ordering::Acquire)
            .try_into()
            .expect("invalid state")
    }

    pub fn set(&self, state: State) {
        self.0.store(state as u8, Ordering::Release);
    }

    pub fn lock(&self, expect: State) -> Result<StateGuard, State> {
        match self.0.compare_exchange(
            expect as u8,
            State::Busy as u8,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(StateGuard(self, expect as u8)),
            Err(old) => Err(old.try_into().expect("invalid state")),
        }
    }
}

#[must_use]
pub struct StateGuard<'a>(&'a StateLock, u8);
impl StateGuard<'_> {
    pub fn transit<R>(self, new: State, f: impl FnOnce() -> AxResult<R>) -> AxResult<R> {
        match f() {
            Ok(result) => {
                self.0.0.store(new as u8, Ordering::Release);
                Ok(result)
            }
            Err(err) => {
                self.0.0.store(self.1, Ordering::Release);
                Err(err)
            }
        }
    }
}
