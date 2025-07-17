//! Coroutine APIs for multi-task configuration.

extern crate alloc;

use crate::io;
use alloc::{string::String, sync::Arc};
use arceos_api::task::{self as api, AxTaskHandle};
use axerrno::ax_err_type;
use core::{cell::UnsafeCell, future::Future, num::NonZeroU64};

/// A unique identifier for a running coroutine task.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct TaskId(NonZeroU64);

/// A handle to a coroutine.
pub struct Task {
    id: TaskId,
}

impl TaskId {
    /// This returns a numeric identifier for the coroutine task identified by this
    /// `TaskId`.
    pub fn as_u64(&self) -> NonZeroU64 {
        self.0
    }
}

impl Task {
    fn from_id(id: u64) -> Self {
        Self {
            id: TaskId(NonZeroU64::new(id).unwrap()),
        }
    }

    /// Gets the coroutine task's unique identifier.
    pub fn id(&self) -> TaskId {
        self.id
    }
}

/// Task factory, which can be used in order to configure the properties of
/// a new coroutine task.
///
/// Methods can be chained on it in order to configure it.
#[derive(Debug)]
pub struct Builder {
    // A name for the coroutine task-to-be, for identification in panic messages
    name: Option<String>,
}

impl Builder {
    /// Generates the base configuration for spawning a coroutine task, from which
    /// configuration methods can be chained.
    pub const fn new() -> Builder {
        Builder { name: None }
    }

    /// Names the coroutine task-to-be.
    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    /// Spawns a new coroutine task by taking ownership of the `Builder`, and returns an
    /// [`io::Result`] to its [`JoinHandle`].
    ///
    /// The spawned coroutine task may outlive the caller (unless the caller coroutine task
    /// is the main coroutine task; the whole process is terminated when the main
    /// coroutine task finishes). The join handle can be used to block on
    /// termination of the spawned coroutine task.
    pub fn spawn<F1, F2, T>(self, f: F1) -> io::Result<JoinHandle<T>>
    where
        F1: FnOnce() -> F2,
        F1: Send + 'static,
        F2: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        unsafe { self.spawn_unchecked(f) }
    }

    unsafe fn spawn_unchecked<F1, F2, T>(self, f: F1) -> io::Result<JoinHandle<T>>
    where
        F1: FnOnce() -> F2,
        F1: Send + 'static,
        F2: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let name = self.name.unwrap_or_default();

        let my_packet = Arc::new(Packet {
            result: UnsafeCell::new(None),
        });
        let their_packet = my_packet.clone();

        let main = async move {
            let ret = f().await;
            // SAFETY: `their_packet` as been built just above and moved by the
            // closure (it is an Arc<...>) and `my_packet` will be stored in the
            // same `JoinHandle` as this closure meaning the mutation will be
            // safe (not modify it and affect a value far away).
            unsafe { *their_packet.result.get() = Some(ret) };
            drop(their_packet);
        };

        let task = api::ax_spawn_f(main, name);
        Ok(JoinHandle {
            task: Task::from_id(task.id()),
            native: task,
            packet: my_packet,
        })
    }
}

/// Gets a handle to the coroutine task that invokes it.
pub fn current() -> Task {
    let id = api::ax_current_task_id();
    Task::from_id(id)
}

/// Spawns a new coroutine task, returning a [`JoinHandle`] for it.
///
/// The join handle provides a [`join`] method that can be used to join the
/// spawned coroutine task.
///
/// The default task name is an empty string. The default coroutine task stack size is
/// [`arceos_api::config::TASK_STACK_SIZE`].
///
/// [`join`]: JoinHandle::join
pub fn spawn<F1, F2, T>(f: F1) -> JoinHandle<T>
where
    F1: FnOnce() -> F2,
    F1: Send + 'static,
    F2: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Builder::new()
        .spawn(f)
        .expect("failed to spawn coroutine task")
}

struct Packet<T> {
    result: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for Packet<T> {}

/// An owned permission to join on a coroutine task (block on its termination).
///
/// A `JoinHandle` *detaches* the associated coroutine task when it is dropped, which
/// means that there is no longer any handle to the coroutine task and no way to `join`
/// on it.
pub struct JoinHandle<T> {
    native: AxTaskHandle,
    task: Task,
    packet: Arc<Packet<T>>,
}

unsafe impl<T> Send for JoinHandle<T> {}
unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    /// Extracts a handle to the underlying coroutine task.
    pub fn task(&self) -> &Task {
        &self.task
    }

    /// Waits for the associated coroutine task to finish.
    ///
    /// This function will return immediately if the associated coroutine task has
    /// already finished.
    pub async fn join_f(mut self) -> io::Result<T> {
        api::ax_wait_for_exit_f(self.native)
            .await
            .ok_or_else(|| ax_err_type!(BadState))?;
        Arc::get_mut(&mut self.packet)
            .unwrap()
            .result
            .get_mut()
            .take()
            .ok_or_else(|| ax_err_type!(BadState))
    }

    /// Waits for the associated coroutine task to finish.
    ///
    /// This function will return immediately if the associated coroutine task has
    /// already finished.
    pub fn join(mut self) -> io::Result<T> {
        api::ax_wait_for_exit(self.native).ok_or_else(|| ax_err_type!(BadState))?;
        Arc::get_mut(&mut self.packet)
            .unwrap()
            .result
            .get_mut()
            .take()
            .ok_or_else(|| ax_err_type!(BadState))
    }
}
