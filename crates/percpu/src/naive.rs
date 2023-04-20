/// No effect for "sp-naive" use.
pub fn init(_max_cpu_num: usize) {}

/// Always returns `0` for "sp-naive" use.
pub fn get_local_thread_pointer() -> usize {
    0
}

/// No effect for "sp-naive" use.
pub fn set_local_thread_pointer(_cpu_id: usize) {}
