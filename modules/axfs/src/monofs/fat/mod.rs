mod dir;
mod ff;
mod file;
mod fs;
mod util;

use core::cell::UnsafeCell;

use fatfs::SeekFrom;
pub use fs::FatFilesystem;
use fs::FatFilesystemInner;

use crate::disk::SeekableDisk;

impl fatfs::IoBase for SeekableDisk {
    type Error = ();
}

impl fatfs::Read for SeekableDisk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        SeekableDisk::read(self, buf).map_err(|_| ())
    }
}

impl fatfs::Write for SeekableDisk {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        SeekableDisk::write(self, buf).map_err(|_| ())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        SeekableDisk::flush(self).map_err(|_| ())
    }
}

impl fatfs::Seek for SeekableDisk {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let size = self.size();
        let new_pos = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.position().checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or(())?;
        self.set_position(new_pos).map_err(|_| ())?;
        Ok(new_pos)
    }
}

/// A reference to an object within a filesystem.
pub(crate) struct FsRef<T> {
    inner: UnsafeCell<T>,
}

impl<T> FsRef<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn borrow<'a>(&self, _fs: &'a FatFilesystemInner) -> &'a T {
        // SAFETY: The filesystem outlives the reference
        unsafe { &*self.inner.get() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn borrow_mut<'a>(&self, _fs: &'a FatFilesystemInner) -> &'a mut T {
        // SAFETY: The filesystem outlives the reference
        unsafe { &mut *self.inner.get() }
    }
}
