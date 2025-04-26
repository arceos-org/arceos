use crate::arch::{TrapFrame, read_thread_pointer, write_thread_pointer};

#[cfg(feature = "tls")]
#[unsafe(no_mangle)]
#[percpu::def_percpu]
static KERNEL_FS_BASE: usize = 0;

pub fn switch_to_kernel_fs_base(tf: &mut TrapFrame) {
    if tf.is_user() {
        tf.fs_base = read_thread_pointer() as _;
        #[cfg(feature = "tls")]
        unsafe {
            write_thread_pointer(KERNEL_FS_BASE.read_current())
        };
    }
}

pub fn switch_to_user_fs_base(tf: &TrapFrame) {
    if tf.is_user() {
        #[cfg(feature = "tls")]
        KERNEL_FS_BASE.write_current(read_thread_pointer());
        unsafe { write_thread_pointer(tf.fs_base as _) };
    }
}
