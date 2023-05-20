use bitflags::bitflags;

bitflags! {
    pub struct OpenFlags: usize {
        /// O_APPEND
        const APPEND = 1 << 0;
        /// O_CREAT
        const CREATE = 1 << 1;
        /// O_TRUNC
        const TRUNCATE = 1 << 2;
    }
}
