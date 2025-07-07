use core::{
    cell::OnceCell,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use axsync::Mutex;
use axtask::yield_now;

use embassy_executor::{SendSpawner, Spawner};

/// Global spawner for multi-thread executor
pub(crate) static SPAWNER: Mutex<OnceCell<SendSpawner>> = Mutex::new(OnceCell::new());

fn init_spawn() {
    use axtask::spawn_raw;
    spawn_raw(init, "async".into(), axconfig::TASK_STACK_SIZE);
}

fn init() {
    use crate::executor::Executor;
    use static_cell::StaticCell;

    static EXECUTOR: StaticCell<Executor> = StaticCell::new();
    EXECUTOR
        .init_with(Executor::new)
        .run(|sp| sp.must_spawn(init_task()));
}

#[embassy_executor::task]
async fn init_task() {
    use crate::asynch;

    let spawner = asynch::Spawner::for_current_executor().await;
    asynch::set_spawner(spawner.make_send());
    log::info!("Initialize spawner... ");
}

/// # Panics
///
/// Panics if the system executor is not initialized.
pub fn spawner() -> SendSpawner {
    let sp = SPAWNER.lock();
    if let Some(inner) = sp.get() {
        *inner
    } else {
        drop(sp);
        init_spawn();
        yield_now();
        // initialize the spawner if not
        let sp = SPAWNER.lock();
        *sp.get().expect("Reinitialize the spawner failed")
    }
}

/// Set the spawner for the system executor.
///
/// May only be called once.
pub(crate) fn set_spawner(spawner: SendSpawner) {
    let sp = SPAWNER.lock();
    let _ = sp.set(spawner);
}

fn wake(_ctx: *const ()) {
    // let id = ctx as u64;
    // unpark_task(id, true);
}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(|ctx| RawWaker::new(ctx, &VTABLE), wake, wake, wake);

/// Panics if not called in a thread task
pub fn block_on<F: Future>(mut fut: F) -> F::Output {
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

    let id = axtask::current().id().as_u64();
    let raw_waker = RawWaker::new(id as *const (), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut ctx = Context::from_waker(&waker);

    loop {
        if let Poll::Ready(res) = fut.as_mut().poll(&mut ctx) {
            return res;
        }
        yield_now();
    }
}
