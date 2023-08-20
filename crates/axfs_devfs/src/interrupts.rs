use alloc::{collections::BTreeMap, format, string::String};
use axerrno::AxError;
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Default)]
pub struct InterruptCounter(BTreeMap<usize, usize>);

impl InterruptCounter {
    pub fn record(&mut self, key: usize) {
        self.0.entry(key).and_modify(|cnt| *cnt += 1).or_insert(1);
    }

    pub fn content(&self) -> String {
        let mut content = String::new();

        for line in self
            .0
            .iter()
            .map(|(key, value)| format!("{}: {}", key, value))
        {
            content.push_str(&line);
            content.push('\n');
        }

        content
    }
}

lazy_static! {
    pub static ref INTERRUPT: Mutex<InterruptCounter> =
        Mutex::new(InterruptCounter(BTreeMap::default()));
}

#[derive(Default)]
pub struct Interrupts;

impl VfsNodeOps for Interrupts {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let content = INTERRUPT.lock().content();
        let bytes = &content.as_bytes();

        let offset = offset as usize;
        if offset > bytes.len() {
            return Err(AxError::InvalidInput);
        }

        let len = if buf.len() < bytes.len() - offset {
            buf.len()
        } else {
            bytes.len() - offset
        };

        buf[..len].copy_from_slice(&bytes[offset..len + offset]);

        Ok(len)
    }

    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(AxError::Io)
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        Err(AxError::Io)
    }

    axfs_vfs::impl_vfs_non_dir_default! {}
}
