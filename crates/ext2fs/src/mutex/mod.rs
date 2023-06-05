#![allow(unused)]
use fs_utils::sync::{self, Spin};

pub type RwSpinMutex<T> = sync::rw_spin_mutex::RwSpinMutex<T, Spin>;
pub type SpinMutex<T> = sync::spin_mutex::SpinMutex<T, Spin>;
