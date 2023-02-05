use alloc::sync::Arc;
use core::{cell::UnsafeCell, ops::Deref};

use linked_list::{Cursor, LinkedList};

use crate::BaseScheduler;

pub struct FifoTask<T> {
    inner: T,
    // `list_cursor` can only be accessed when holding the mutable reference
    // of the list, thus we can use the `UnsafeCell`.
    list_cursor: UnsafeCell<Cursor<Arc<FifoTask<T>>>>,
}

unsafe impl<T: Send> Send for FifoTask<T> {}
unsafe impl<T: Sync> Sync for FifoTask<T> {}

impl<T> FifoTask<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            list_cursor: UnsafeCell::new(Cursor::default()),
        }
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }

    const fn list_cursor(&self) -> &Cursor<Arc<FifoTask<T>>> {
        unsafe { &*self.list_cursor.get() }
    }

    #[allow(clippy::mut_from_ref)]
    const fn list_cursor_mut(&self) -> &mut Cursor<Arc<FifoTask<T>>> {
        unsafe { &mut *self.list_cursor.get() }
    }
}

impl<T> const Deref for FifoTask<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct FifoScheduler<T> {
    ready_queue: LinkedList<Arc<FifoTask<T>>>,
}

impl<T> FifoScheduler<T> {
    pub const fn new() -> Self {
        Self {
            ready_queue: LinkedList::new(),
        }
    }
}

impl<T> BaseScheduler for FifoScheduler<T> {
    type SchedItem = Arc<FifoTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        self.ready_queue.push_back(task);
        *self.ready_queue.back().unwrap().list_cursor_mut() = self.ready_queue.cursor_back();
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        self.ready_queue.remove_cursor(task.list_cursor_mut())
    }

    fn yield_task(&mut self, task: &Self::SchedItem) {
        // remove the task and then push it to the back, without deallocating the node.
        if let Some(mut new_list) = self.ready_queue.remove_cursor_as_list(task.list_cursor()) {
            self.ready_queue.append(&mut new_list);
            // task.list_cursor was unchanged
        }
    }

    fn pick_next_task(&self) -> Option<&Self::SchedItem> {
        self.ready_queue.front()
    }

    fn task_tick(&mut self, _current: &Self::SchedItem) -> bool {
        false // no reschedule
    }
}
