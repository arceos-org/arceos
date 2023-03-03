#![no_std]

pub type Mutex<T> = spinlock::SpinNoIrq<T>; // TODO
