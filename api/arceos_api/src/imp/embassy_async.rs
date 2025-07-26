cfg_async! {
    pub use axembassy::Executor as AxExecutor;
    pub use axembassy::Spawner as AxSpawner;
}

cfg_async_thread! {
    pub use axembassy::SendSpawner as AxSendSpawner;

    pub fn ax_spawner() -> AxSendSpawner {
        axembassy::spawner()
    }

    pub fn ax_block_on<F: Future>(fut: F) -> F::Output {
        axembassy::block_on(fut)
    }
}

cfg_async_preempt! {
    pub use axembassy::PrioFuture as AxPrioFuture;
}
