use bitflags::*;

bitflags! {
    /// OpenFlags
    /// TODO(weny): Adds comments
    pub struct OpenFlags: u32 {
        const RDONLY = 0;

        const WRONLY = 1 << 0;

        const RDWR = 1 << 1;

        const CREATE = 1 << 6;

        const EXCLUSIVE = 1 << 7;

        const NOCTTY = 1 << 8;

        const EXCL = 1 << 9;

        const NON_BLOCK = 1 << 11;

        const TEXT = 1 << 14;

        const BINARY = 1 << 15;

        const DSYNC = 1 << 16;

        const NOFOLLOW = 1 << 17;

        const CLOEXEC = 1 << 19;

        const DIR = 1 << 21;
    }
}

impl OpenFlags {
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }

    pub fn readable(&self) -> bool {
        !self.contains(Self::WRONLY)
    }

    pub fn writable(&self) -> bool {
        self.contains(Self::WRONLY) || self.contains(Self::RDWR)
    }
}
