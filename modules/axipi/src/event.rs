use alloc::{boxed::Box, sync::Arc};

/// A callback function that will be called when an [`IPIEvent`] is received and handled.
pub struct Callback(Box<dyn FnOnce()>);

impl Callback {
    /// Create a new [`Callback`] with the given function.
    pub fn new<F: FnOnce() + 'static>(callback: F) -> Self {
        Self(Box::new(callback))
    }

    /// Call the callback function.
    pub fn call(self) {
        (self.0)()
    }
}

impl<T: FnOnce() + 'static> From<T> for Callback {
    fn from(callback: T) -> Self {
        Self::new(callback)
    }
}

/// A [`Callback`] that can be called multiple times. It's used for multicast IPI events.
#[derive(Clone)]
pub struct MulticastCallback(Arc<dyn Fn()>);

impl MulticastCallback {
    /// Create a new [`MulticastCallback`] with the given function.
    pub fn new<F: Fn() + 'static>(callback: F) -> Self {
        Self(Arc::new(callback))
    }

    /// Convert the [`MulticastCallback`] into a [`Callback`].
    pub fn into_unicast(self) -> Callback {
        Callback(Box::new(move || {
            (self.0)()
        }))
    }
}

impl<T: Fn() + 'static> From<T> for MulticastCallback {
    fn from(callback: T) -> Self {
        Self::new(callback)
    }
}

/// An IPI event that is sent from a source CPU to the target CPU.
pub struct IPIEvent {
    pub src_cpu_id: usize,
    pub callback: Callback,
}