use alloc::boxed::Box;
use axfs::api::{File, FileExt};
use axio::{Read, Seek, SeekFrom};

/// File backend for Lazy load `MapArea`. `file` should be a file holding a offset value. Normally,
/// `MemBackend` won't share a file with other things, so we use a `Box` here.
pub struct MemBackend {
    file: Box<dyn FileExt>,
}

impl MemBackend {
    pub fn new(mut file: Box<dyn FileExt>, offset: u64) -> Self {
        let _ = file.seek(SeekFrom::Start(offset)).unwrap();

        Self { file }
    }

    pub fn clone_with_delta(&self, delta: i64) -> Self {
        let mut new_backend = self.clone();

        let _ = new_backend.seek(SeekFrom::Current(delta)).unwrap();

        new_backend
    }

    pub fn read_from_seek(&mut self, pos: SeekFrom, buf: &mut [u8]) -> Result<usize, axio::Error> {
        self.file.read_from_seek(pos, buf)
    }

    pub fn write_to_seek(&mut self, pos: SeekFrom, buf: &[u8]) -> Result<usize, axio::Error> {
        self.file.write_to_seek(pos, buf)
    }

    pub fn readable(&self) -> bool {
        self.file.readable()
    }

    pub fn writable(&self) -> bool {
        self.file.writable()
    }
}

impl Clone for MemBackend {
    fn clone(&self) -> Self {
        let file = self
            .file
            .as_any()
            .downcast_ref::<File>()
            .expect("Cloning a MemBackend with a non-file object")
            .clone();

        Self {
            file: Box::new(file),
        }
    }
}

impl Seek for MemBackend {
    fn seek(&mut self, pos: SeekFrom) -> axio::Result<u64> {
        self.file.seek(pos)
    }
}

impl Read for MemBackend {
    fn read(&mut self, buf: &mut [u8]) -> axio::Result<usize> {
        self.file.read(buf)
    }
}
