use alloc::collections::VecDeque;

use crate::{Callback, IPIEvent};

/// A queue of IPI events.
///
/// It internally uses a `VecDeque` to store the events, make it
/// possible to pop these events using FIFO order.
pub struct IPIEventQueue {
    events: VecDeque<IPIEvent>,
}

impl IPIEventQueue {
    /// Create a new empty timer list.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Whether there is no event.
    #[allow(dead_code)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Push a new event into the queue.
    pub fn push(&mut self, src_cpu_id: usize, callback: Callback) {
        self.events.push_back(IPIEvent { src_cpu_id, callback });
    }

    /// Try to pop the latest event that exists in the queue.
    ///
    /// Return `None` if no event is available.
    #[must_use]
    pub fn pop_one(&mut self) -> Option<(usize, Callback)> {
        if let Some(e) = self.events.pop_front() {
            Some((e.src_cpu_id, e.callback))
        } else {
            None
        }
    }
}

impl Default for IPIEventQueue {
    fn default() -> Self {
        Self::new()
    }
}
