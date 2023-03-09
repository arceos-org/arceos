struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn task_try_preempt() {
        #[cfg(feature = "multitask")]
        axtask::try_preempt();
    }
}
