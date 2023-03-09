#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

pub type Handler = fn();

pub struct HandlerTable<const N: usize> {
    handlers: [AtomicUsize; N],
}

impl<const N: usize> HandlerTable<N> {
    #[allow(clippy::declare_interior_mutable_const)]
    pub const fn new() -> Self {
        const EMPTY: AtomicUsize = AtomicUsize::new(0);
        Self {
            handlers: [EMPTY; N],
        }
    }

    pub fn register_handler(&self, idx: usize, handler: Handler) -> bool {
        self.handlers[idx]
            .compare_exchange(0, handler as usize, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn handle(&self, idx: usize) -> bool {
        let handler = self.handlers[idx].load(Ordering::Acquire);
        if handler != 0 {
            let handler: Handler = unsafe { core::mem::transmute(handler) };
            handler();
            true
        } else {
            false
        }
    }
}
