//! The system abstraction layer.
use core::{any::Any, cell::RefCell};
extern crate alloc;
use alloc::{
    boxed::Box,
    rc::{Rc, Weak},
};

use crate::tx::{Handle, JournalBuffer};

pub trait BlockDevice: Any {
    /// Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
    /// Block size of the device
    fn block_size(&self) -> usize;
}

pub trait Buffer: Any {
    // fn device(&self) -> Rc<dyn BlockDevice>;
    fn block_id(&self) -> usize;
    fn size(&self) -> usize;
    fn dirty(&self) -> bool;
    fn data(&self) -> *mut u8;

    // Related methods of the `private` field of `struct buffer_head`
    fn private(&self) -> &Option<Box<dyn Any>>;
    fn set_private(&self, private: Option<Box<dyn Any>>);

    // Normal writeback control. JBD might alter the related states
    // to control writeback behaviours.
    fn mark_dirty(&self);
    fn clear_dirty(&self);
    fn test_clear_dirty(&self) -> bool;
    fn sync(&self);

    // JBD-specific state management. The related states should only
    // be altered by JBD.
    fn jbd_managed(&self) -> bool;
    fn set_jbd_managed(&self, managed: bool);
    fn mark_jbd_dirty(&self);
    fn clear_jbd_dirty(&self);
    fn test_clear_jbd_dirty(&self) -> bool;
    fn jbd_dirty(&self) -> bool;
    fn revoked(&self) -> bool;
    fn set_revoked(&self);
    fn test_set_revoked(&self) -> bool;
    fn clear_revoked(&self);
    fn test_clear_revoked(&self) -> bool;
    fn revoke_valid(&self) -> bool;
    fn set_revoke_valid(&self);
    fn test_set_revoke_valid(&self) -> bool;
    fn clear_revoke_valid(&self);
    fn test_clear_revoke_valid(&self) -> bool;
}

impl dyn Buffer {
    pub(crate) fn buf(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data(), self.size()) }
    }

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

    pub(crate) fn convert_mut<T>(&self) -> &mut T {
        self.mark_dirty();
        unsafe { &mut *(self.data() as *mut T) }
    }

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

pub trait BufferProvider: Any {
    fn get_buffer(&self, dev: &Rc<dyn BlockDevice>, block_id: usize) -> Option<Rc<dyn Buffer>>;
}

pub trait System: Any {
    fn get_buffer_provider(&self) -> Rc<dyn BufferProvider>;
    fn get_time(&self) -> usize;
    fn get_current_handle(&self) -> Option<Rc<RefCell<Handle>>>;
    fn set_current_handle(&self, handle: Option<Rc<RefCell<Handle>>>);
}
