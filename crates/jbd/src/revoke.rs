use core::{cell::RefCell, mem::size_of};

extern crate alloc;
use alloc::{collections::BTreeMap, rc::Rc};

use crate::{
    config::JFS_MAGIC_NUMBER,
    disk::{BlockType, Header, RevokeBlockHeader},
    err::{JBDError, JBDResult},
    journal::JournalFlag,
    sal::Buffer,
    tx::{JournalBuffer, Tid, Transaction},
    Handle, Journal,
};

#[cfg(feature = "debug")]
use crate::disk::Display;

#[derive(Clone, Copy)]
pub(crate) struct RevokeRecord {
    sequence: Tid,
    blocknr: u32,
}

impl Handle {
    pub fn revoke(&mut self, buf: &Rc<dyn Buffer>) -> JBDResult {
        let transcation_rc = self.transaction.as_ref().unwrap().clone();
        let mut transaction = transcation_rc.as_ref().borrow_mut();
        let journal_rc = transaction.journal.upgrade().unwrap();
        let mut journal = journal_rc.as_ref().borrow_mut();

        if buf.revoked() {
            log::error!(
                "Buffer {} is revoked again; data is inconsistent!",
                buf.block_id()
            );
            return Err(JBDError::IOError);
        }

        buf.set_revoked();
        buf.set_revoke_valid();

        self.forget(buf, &transcation_rc, &mut transaction)?;

        journal.insert_revoke_record(buf.block_id() as u32, transaction.tid);

        log::debug!(
            "Revoked buffer {} in transaction {}",
            buf.block_id(),
            transaction.tid
        );

        Ok(())
    }

    pub(crate) fn cancel_revoke(&self, jb: &JournalBuffer) -> JBDResult {
        let buf = &jb.buf;
        let transaction_rc = jb.transaction.as_ref().unwrap();
        let transaction = transaction_rc.borrow();
        let journal_rc = transaction.journal.upgrade().unwrap();
        let mut journal = journal_rc.as_ref().borrow_mut();

        log::debug!("Canceling revoke for buffer {}", buf.block_id());

        let need_cancel = if buf.test_set_revoke_valid() {
            buf.test_clear_revoked()
        } else {
            buf.clear_revoked();
            true
        };

        if need_cancel
            && journal
                .get_revoke_table_mut()
                .remove_entry(&(buf.block_id() as u32))
                .is_some()
        {
            log::debug!(
                "Canceling revoke for buffer {} in transaction {}",
                buf.block_id(),
                transaction.tid
            );
        }

        Ok(())
    }
}

impl Journal {
    fn insert_revoke_record(&mut self, blocknr: u32, sequence: Tid) {
        let record = RevokeRecord { sequence, blocknr };
        self.get_revoke_table_mut().insert(blocknr, record);
    }

    pub(crate) fn switch_revoke_table(&mut self) {
        if self.current_revoke_table == 0 {
            self.current_revoke_table = 1;
        } else {
            self.current_revoke_table = 0;
        }
    }

    pub(crate) fn write_revoke_records(&mut self, transaction: &mut Transaction) -> JBDResult {
        let mut descriptor_rc: Option<Rc<RefCell<JournalBuffer>>> = None;

        let revoke_table = if self.current_revoke_table == 0 {
            let ret = self.revoke_tables[1].clone();
            self.revoke_tables[1].clear();
            ret
        } else {
            let ret = self.revoke_tables[0].clone();
            self.revoke_tables[0].clear();
            ret
        };

        let count = revoke_table.len();
        let mut offset = 0;

        for (_, record) in revoke_table.into_iter() {
            self.write_one_revoke_record(transaction, &mut descriptor_rc, &mut offset, &record)?;
        }

        if let Some(descriptor_rc) = descriptor_rc {
            self.flush_descriptor(&descriptor_rc.as_ref().borrow(), offset);
        }

        log::debug!("Wrote {} revoke records", count);

        Ok(())
    }

    fn write_one_revoke_record(
        &mut self,
        transaction: &mut Transaction,
        descriptor_rc: &mut Option<Rc<RefCell<JournalBuffer>>>,
        offset: &mut u32,
        record: &RevokeRecord,
    ) -> JBDResult {
        if self.flags.contains(JournalFlag::ABORT) {
            return Ok(());
        }

        if let Some(descriptor) = descriptor_rc {
            let descriptor = descriptor.as_ref().borrow();
            if *offset == descriptor.buf.size() as u32 {
                self.flush_descriptor(&descriptor, *offset);
                drop(descriptor);
                *descriptor_rc = None;
            }
        }

        if descriptor_rc.is_none() {
            *descriptor_rc = Some(self.get_descriptor_buffer()?);
            let descriptor = descriptor_rc.as_ref().unwrap().as_ref().borrow_mut();
            let header: &mut Header = descriptor.buf.convert_mut();
            header.magic = JFS_MAGIC_NUMBER.to_be();
            header.block_type = BlockType::RevokeBlock.to_u32_be();
            header.sequence = (transaction.tid as u32).to_be();

            #[cfg(feature = "debug")]
            log::debug!("Added descriptor: {}", header.display(0));

            descriptor.buf.sync();

            *offset = size_of::<RevokeBlockHeader>() as u32;
        }

        let descriptor = descriptor_rc.as_ref().unwrap().as_ref().borrow_mut();
        let blocknr = descriptor.buf.convert_offset_mut::<u32>(*offset as usize);
        *blocknr = record.blocknr;
        *offset += 4;

        #[cfg(feature = "debug")]
        log::debug!("Added revoke tag: {}", record.blocknr);

        Ok(())
    }

    pub(crate) fn clear_buffer_revoked_flags(&mut self) {
        for (blocknr, _) in self.get_revoke_table().iter() {
            if let Ok(buf) = self.get_buffer_translated(*blocknr) {
                buf.clear_revoked();
            }
        }
    }

    pub(crate) fn clear_revoke(&mut self) {
        self.get_revoke_table_mut().clear();
    }

    pub(crate) fn set_revoke(&mut self, blocknr: u32, sequence: Tid) {
        let revoke_table = self.get_revoke_table_mut();
        revoke_table
            .entry(blocknr)
            .and_modify(|rec| rec.sequence = sequence.max(rec.sequence))
            .or_insert(RevokeRecord { sequence, blocknr });
    }

    pub(crate) fn test_revoke(&self, blocknr: u32, sequence: Tid) -> bool {
        self.get_revoke_table()
            .get(&blocknr)
            .map_or(false, |rec| rec.sequence >= sequence)
    }

    fn flush_descriptor(&mut self, descriptor: &JournalBuffer, offset: u32) {
        let header: &mut RevokeBlockHeader = descriptor.buf.convert_mut();
        header.count = offset.to_be();
        descriptor.buf.sync();
    }

    fn get_revoke_table(&self) -> &BTreeMap<u32, RevokeRecord> {
        &self.revoke_tables[self.current_revoke_table]
    }

    fn get_revoke_table_mut(&mut self) -> &mut BTreeMap<u32, RevokeRecord> {
        &mut self.revoke_tables[self.current_revoke_table]
    }
}
