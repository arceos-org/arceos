use bitflags::bitflags;

bitflags! {
    pub struct OpenFlags: usize {
        /// O_APPEND
        const APPEND = 1 << 0;
        /// O_CREAT
        const CREATE = 1 << 1;
        /// O_TRUNC
        const TRUNCATE = 1 << 2;
        /// O_RDONLY (O_RDWR)
        const READ = 1 << 3;
        /// O_WRONLY (O_RDWR)
        const WRITE = 1 << 4;
        /// O_DIRECTORY
        const DIRECTORY = 1 << 5;
        /// O_EXCL
        const EXCL = 1 << 6;
    }
}

pub const SEEK_SET: usize = 0;
pub const SEEK_CUR: usize = 1;
pub const SEEK_END: usize = 2;
