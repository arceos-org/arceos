//! The system abstraction layer.
use core::{any::Any, cell::RefCell};
extern crate alloc;
use alloc::{
    boxed::Box,
    rc::{Rc, Weak},
};

use crate::tx::{Handle, JournalBuffer};

/// The block device trait.
pub trait BlockDevice: Any {
    /// Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
    /// Block size of the device
    fn block_size(&self) -> usize;
}

/// The buffer for a block.
pub trait Buffer: Any {
    /// The block id of the buffer.
    fn block_id(&self) -> usize;
    /// The size of the buffer.
    fn size(&self) -> usize;
    /// Whether the buffer is dirty.
    fn dirty(&self) -> bool;
    /// The data pointer of the buffer.
    fn data(&self) -> *mut u8;

    // Related methods of the `private` field of `struct buffer_head`
    /// The private field of the buffer. Returns the reference of the value set by `set_private`.
    fn private(&self) -> &Option<Box<dyn Any>>;
    /// Set the private field of the buffer.
    fn set_private(&self, private: Option<Box<dyn Any>>);

    // Normal writeback control. JBD might alter the related states
    // to control writeback behaviours.
    /// Mark the buffer as dirty.
    fn mark_dirty(&self);
    /// Clear the dirty flag of the buffer.
    fn clear_dirty(&self);
    /// Test and clear the dirty flag of the buffer.
    fn test_clear_dirty(&self) -> bool;
    /// Sync the buffer.
    fn sync(&self);

    // JBD-specific state management. The related states should only
    // be altered by JBD.
    /// Whether the buffer is managed by JBD.
    fn jbd_managed(&self) -> bool;
    /// Set the buffer as managed or not by JBD.
    fn set_jbd_managed(&self, managed: bool);
    /// Mark the buffer as internally dirty by JBD.
    fn mark_jbd_dirty(&self);
    /// Clear the JBD dirty flag of the buffer.
    fn clear_jbd_dirty(&self);
    /// Test and clear the JBD dirty flag of the buffer.
    fn test_clear_jbd_dirty(&self) -> bool;
    /// Whether the buffer is internally dirty by JBD.
    fn jbd_dirty(&self) -> bool;
    /// Whether the buffer is revoked.
    fn revoked(&self) -> bool;
    /// Set the buffer as revoked.
    fn set_revoked(&self);
    /// Test and set the buffer as revoked.
    fn test_set_revoked(&self) -> bool;
    /// Clear the revoked flag of the buffer.
    fn clear_revoked(&self);
    /// Test and clear the revoked flag of the buffer.
    fn test_clear_revoked(&self) -> bool;
    /// Whether the revoke state is valid.
    fn revoke_valid(&self) -> bool;
    /// Set the revoke state as valid.
    fn set_revoke_valid(&self);
    /// Test and set the revoke state as valid.
    fn test_set_revoke_valid(&self) -> bool;
    /// Clear the revoke state as valid.
    fn clear_revoke_valid(&self);
    /// Test and clear the revoke state as valid.
    fn test_clear_revoke_valid(&self) -> bool;
}

impl dyn Buffer {
    pub(crate) fn buf(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data(), self.size()) }
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn buf_mut(&self) -> &mut [u8] {
        self.mark_dirty();
        unsafe { core::slice::from_raw_parts_mut(self.data(), self.size()) }
    }

    pub(crate) fn convert<T>(&self) -> &T {
        unsafe { &*(self.data() as *const T) }
    }

    pub(crate) fn convert_offset<T>(&self, offset: usize) -> &T {
        unsafe { &*((self.data() as usize + offset) as *const T) }
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn convert_mut<T>(&self) -> &mut T {
        self.mark_dirty();
        unsafe { &mut *(self.data() as *mut T) }
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn convert_offset_mut<T>(&self, offset: usize) -> &mut T {
        self.mark_dirty();
        unsafe { &mut *((self.data() as usize + offset) as *mut T) }
    }

    pub(crate) fn journal_buffer(&self) -> Option<Rc<RefCell<JournalBuffer>>> {
        let private = self.private();
        match private {
            None => None,
            Some(x) => match x.downcast_ref::<Weak<RefCell<JournalBuffer>>>() {
                None => None,
                Some(x) => x.upgrade(),
            },
        }
    }

    pub(crate) fn set_journal_buffer(&self, jb: Rc<RefCell<JournalBuffer>>) {
        self.set_jbd_managed(true);
        self.set_private(Some(Box::new(Rc::downgrade(&jb))));
    }

    pub(crate) fn clear_journal_buffer(&self) {
        self.set_jbd_managed(false);
        self.set_private(None);
    }
}

/// A provider (manager) of buffers.
pub trait BufferProvider: Any {
    /// Get the buffer for a block.
    fn get_buffer(&self, dev: &Rc<dyn BlockDevice>, block_id: usize) -> Option<Rc<dyn Buffer>>;
}

/// The interfaces of the abstract system.
pub trait System: Any {
    /// Returns a buffer provider.
    fn get_buffer_provider(&self) -> Rc<dyn BufferProvider>;
    /// Returns the current time.
    fn get_time(&self) -> usize;
    /// Returns the current handle set by `set_current_handle`.
    fn get_current_handle(&self) -> Option<Rc<RefCell<Handle>>>;
    /// Set the current handle.
    fn set_current_handle(&self, handle: Option<Rc<RefCell<Handle>>>);
}
