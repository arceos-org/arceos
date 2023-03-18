#[percpu::def_percpu]
static CPU_ID: usize = 0;

#[percpu::def_percpu]
static IS_BSP: bool = false;

#[percpu::def_percpu]
static CURRENT_TASK_PTR: usize = 0;

#[inline]
pub fn this_cpu_id() -> usize {
    CPU_ID.read_current()
}

#[inline]
pub fn this_cpu_is_bsp() -> bool {
    IS_BSP.read_current()
}

#[allow(dead_code)]
pub(crate) fn init_percpu(cpu_id: usize, is_bsp: bool) {
    if is_bsp {
        percpu::init(axconfig::SMP);
    }
    percpu::set_local_thread_pointer(cpu_id, None);
    unsafe {
        // preemption is disabled on initialization.
        CPU_ID.write_current_raw(cpu_id);
        IS_BSP.write_current_raw(is_bsp);
    }
}
