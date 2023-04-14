pub fn yield_now() {
    axhal::arch::wait_for_irqs();
}

pub fn exit(exit_code: i32) -> ! {
    debug!("main task exited: exit_code={}", exit_code);
    axhal::misc::terminate()
}

pub fn sleep(dur: core::time::Duration) {
    let deadline = axhal::time::current_time() + dur;
    sleep_until(deadline)
}

pub fn sleep_until(deadline: axhal::time::TimeValue) {
    while axhal::time::current_time() < deadline {
        core::hint::spin_loop();
    }
}
