//! Async Standard library macros

/// The `block_on!` macro which is used to poll a async function in a busy spinning manner.
///
/// This macro is used in the normal function. In the async function, the future can be direcyly invoked.
///
/// Usage scenarios: when the future can be polled without waiting or the waiting for a short time, it should use this macro.
///
/// Examples:
///
/// ```rust
/// fn main() {
///     block_on!{hello_world()};
/// }
///
/// async fn hello_world() {
///     println!("hello world!");
/// }
/// ```
///
#[macro_export]
macro_rules! block_on {
    ($l:expr) => {
        // The future can be pinned on the stack directly
        // because the stack cannot be used by other task.
        let mut future = $l;
        let mut pinned_fut = unsafe { core::pin::Pin::new_unchecked(&mut future) };
        // The waker can use the `Waker::noop()` because
        // there is no task switching while polling the future.
        // The task which call this macro and poll this future can
        // be preempt by the timer IRQ.
        let waker = core::task::Waker::noop();
        let mut cx = core::task::Context::from_waker(&waker);
        loop {
            if let core::task::Poll::Ready(res) = pinned_fut.as_mut().poll(&mut cx) {
                break res;
            }
        }
    };
}

/// The `callasync!` macro is the same as the `block_on!`,
/// but it is combined with thread switching.
///
/// This macro is used in the normal function.
/// In the async function, the future can be direcyly invoked.
///
/// Usage scenarios:
///     when the future need wait for a long time to be `Poll::Ready`,
///     and the thread must wait for the result of the future,
///     it should use this macro.
///     It can yield the thread to run other task.
///
/// The yield operation can be defined through a `trait`
/// which is as the same as the implementation in
/// [`axlog`](https://github.com/arceos-org/arceos/tree/main/modules/axlog) crate.
///
/// Examples:
/// ```rust
/// fn main() {
///     callasync!{test()};
/// }
///
/// async fn test() -> i32 {
///     let mut flag = false;
///     core::future::poll_fn(|_cx| {
///         if !flag {
///             flag = true;
///             core::task::Poll::Pending
///         } else {
///             core::task::Poll::Ready(())
///         }
///     }).await;
///     43
/// }
/// ```
#[macro_export]
macro_rules! callasync {
    ($l:expr) => {
        // The future can be pinned on the stack directly
        // because the stack cannot be used by other task.
        let mut future = $l;
        let mut pinned_fut = unsafe { core::pin::Pin::new_unchecked(&mut future) };
        // The waker can use the `Waker::noop()` because
        // the task is switched as a thread.
        // The task which call this macro and poll this future can
        // be preempt by the timer IRQ.
        let waker = core::task::Waker::noop();
        let mut cx = core::task::Context::from_waker(&waker);
        loop {
            match pinned_fut.as_mut().poll(&mut cx) {
                core::task::Poll::Ready(r) => break r,
                core::task::Poll::Pending => {
                    // Yield the task which call this marco when the future return `Pending`.
                    $crate::task::_api::ax_yield_now();
                }
            }
        }
    };
}
