use alloc::collections::VecDeque;

/// A queue of IPI events.
///
/// It internally uses a `VecDeque` to store the events, make it
/// possible to pop these events using FIFO order.
pub struct IPIEventQueue<E: IPIEvent> {
    events: VecDeque<IPIEventWrapper<E>>,
}

/// A trait that all events must implement.
pub trait IPIEvent: 'static {
    /// Callback function that will be called when the event is triggered.
    fn callback(self);
}

struct IPIEventWrapper<E> {
    src_cpu_id: u32,
    event: E,
}

impl<E: IPIEvent> IPIEventQueue<E> {
    /// Creates a new empty timer list.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Whether there is no event.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn push(&mut self, src_cpu_id: u32, event: E) {
        self.events.push_back(IPIEventWrapper { src_cpu_id, event });
    }

    /// Try to pop the latest event that exists in the queue.
    ///
    /// Returns `None` if no event is available.
    pub fn pop_one(&mut self) -> Option<E> {
        if let Some(e) = self.events.pop_front() {
            Some(e.event)
        } else {
            None
        }
    }
}

impl<E: IPIEvent> Default for IPIEventQueue<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple wrapper of a closure that implements the [`IPIEvent`] trait.
///
/// So that it can be used as in the [`IPIEventQueue`].
pub struct IPIEventFn(Box<dyn FnOnce() + 'static>);

impl IPIEventFn {
    /// Constructs a new [`IPIEventFn`] from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        Self(Box::new(f))
    }
}

impl IPIEvent for IPIEventFn {
    fn callback(self) {
        (self.0)()
    }
}
