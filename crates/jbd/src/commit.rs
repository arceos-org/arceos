use core::{cell::RefCell, mem::size_of};

extern crate alloc;
use alloc::{rc::Rc, vec::Vec};

use crate::{
    config::JFS_MAGIC_NUMBER,
    disk::{BlockTag, BlockType, Header, TagFlag},
    err::JBDResult,
    jbd_assert,
    journal::JournalFlag,
    tx::{BufferListType, JournalBuffer, Transaction, TransactionState},
    Journal,
};

#[cfg(feature = "debug")]
use crate::disk::Display;

/// Commit related interfaces.
impl Journal {
    /// Commit the currently-running transaction.
    pub fn commit_transaction(&mut self) -> JBDResult {
        // First job: lock down the current transaction and wait for
        // all outstanding updates to complete.
        if self.flags.contains(JournalFlag::FLUSHED) {
            self.update_superblock();
        }
        jbd_assert!(self.running_transaction.is_some());
        jbd_assert!(self.committing_transaction.is_none());

        let commit_tx_rc = self.running_transaction.as_ref().unwrap().clone();
        let mut commit_tx = commit_tx_rc.as_ref().borrow_mut();
        jbd_assert!(commit_tx.state == TransactionState::Running);

        log::debug!("Start committing transaction {}.", commit_tx.tid);
        commit_tx.state = TransactionState::Locked;

        jbd_assert!(commit_tx.updates == 0);
        jbd_assert!(commit_tx.outstanding_credits <= self.max_transaction_buffers);

        for jb_rc in commit_tx.reserved_list.0.clone().into_iter() {
            let mut jb = jb_rc.as_ref().borrow_mut();
            if jb.commited_data.is_some() {
                jb.commited_data = None;
            }
            Transaction::refile_buffer(&jb_rc, &mut jb, &mut commit_tx);
        }

        // Commit phase 1

        // Clear revoked flag to reflect there is no revoked buffers
        // in the next transaction which is going to be started.
        self.clear_buffer_revoked_flags();
        self.switch_revoke_table();

        commit_tx.state = TransactionState::Flush;
        drop(commit_tx);

        self.committing_transaction = self.running_transaction.clone();
        self.running_transaction = None;
        let commit_tx_rc = self.committing_transaction.as_mut().unwrap().clone();
        let mut commit_tx = commit_tx_rc.as_ref().borrow_mut();
        commit_tx.log_start = self.head;

        // Commit phase 2

        // Now start flushing things to disk, in the order they appear
        // on the transaction lists.  Data blocks go first.
        // self.
        self.submit_data_buffers(&commit_tx_rc, &mut commit_tx)?;

        for jb_rc in commit_tx.locked_list.0.clone().into_iter() {
            let mut jb = jb_rc.as_ref().borrow_mut();
            let buf = &jb.buf;

            if buf.jbd_managed() && jb.jlist == BufferListType::Locked {
                Transaction::unfile_buffer(&jb_rc, &mut jb, &mut commit_tx);
            }
        }

        self.write_revoke_records(&mut commit_tx)?;

        jbd_assert!(commit_tx.sync_datalist.0.is_empty());

        // Commit phase 3

        commit_tx.state = TransactionState::Commit;

        jbd_assert!(commit_tx.nr_buffers <= commit_tx.outstanding_credits as i32);

        let mut descriptor_rc: Option<Rc<RefCell<JournalBuffer>>> = None;
        let mut first_tag = false;
        let mut descriptor_buf_data: Option<*mut u8> = None;
        let mut space_left = 0;

        // Metadata list
        let buffers = commit_tx.buffers.0.clone();
        let buffer_count = buffers.len();

        for (i, jb_rc) in commit_tx.buffers.0.clone().into_iter().enumerate() {
            let mut jb = jb_rc.as_ref().borrow_mut();
            if self.flags.contains(JournalFlag::ABORT) {
                jb.buf.clear_jbd_dirty();
                Transaction::refile_buffer(&jb_rc, &mut jb, &mut commit_tx);
                continue;
            }

            if descriptor_rc.is_none() {
                descriptor_rc = Some(self.get_descriptor_buffer()?);
                let descriptor_rc = descriptor_rc.as_mut().unwrap();
                let descriptor = descriptor_rc.as_ref().borrow_mut();
                let buf = &descriptor.buf;
                let header: &mut Header = buf.convert_mut();
                header.magic = JFS_MAGIC_NUMBER.to_be();
                header.block_type = BlockType::DescriptorBlock.to_u32_be();
                header.sequence = (commit_tx.tid as u32).to_be();

                buf.mark_dirty();
                self.wbuf.push(buf.clone());
                first_tag = true;
                descriptor_buf_data = Some(buf.buf_mut()[size_of::<Header>()..].as_mut_ptr());
                space_left = buf.size() - size_of::<Header>();

                #[cfg(not(feature = "debug"))]
                log::debug!("Added descriptor.");
                #[cfg(feature = "debug")]
                log::debug!("Added descriptor: {}", header.display(0));
            }

            // Where is the buffer to be written?
            let blocknr = self.next_log_block();

            commit_tx.outstanding_credits -= 1;

            let (new_jb_rc, do_escape, _) = self.write_metadata_buffer(
                &commit_tx_rc,
                &mut commit_tx,
                &jb_rc,
                &mut jb,
                blocknr,
            )?;
            self.wbuf.push(new_jb_rc.as_ref().borrow().buf.clone());

            let mut tag_flag = TagFlag::default();
            if do_escape {
                tag_flag.insert(TagFlag::ESCAPE);
            }
            if !first_tag {
                tag_flag.insert(TagFlag::SAME_UUID);
            }

            let tag_ptr = descriptor_buf_data.unwrap();
            let tag_mut = unsafe { &mut *(tag_ptr as *mut BlockTag) };
            tag_mut.block_nr = (jb.buf.block_id() as u32).to_be();
            tag_mut.flag = tag_flag.bits().to_be();
            space_left -= size_of::<BlockTag>();

            #[cfg(feature = "debug")]
            log::debug!("Added block to descriptor: {}", tag_mut.display(0));

            unsafe {
                *descriptor_buf_data.as_mut().unwrap() = descriptor_buf_data
                    .as_ref()
                    .unwrap()
                    .offset(size_of::<BlockTag>() as isize);
            }

            if first_tag {
                // TODO: Fill UUID
                unsafe {
                    *descriptor_buf_data.as_mut().unwrap() =
                        descriptor_buf_data.as_ref().unwrap().offset(16);
                }
                first_tag = false;
                space_left -= 16; // UUID
            }

            // Submit IO
            if i == buffer_count - 1 || space_left < size_of::<BlockTag>() + 16 {
                log::debug!("Submit {} IO.", self.wbuf.len());
                // Write an end-of-descriptor marker before submitting the IOs.
                tag_flag.insert(TagFlag::LAST_TAG);
                tag_mut.flag = tag_flag.bits().to_be();

                #[cfg(feature = "debug")]
                log::debug!("Modified for last tag: {}", tag_mut.display(0));

                for buf in self.wbuf.iter() {
                    buf.mark_dirty();
                    buf.sync();
                }
                self.wbuf.clear();

                descriptor_rc = None;
            }
        }

        // We don't need to wait for the buffer to be written, as they are synced.

        // Commit phase 4-5

        // IO bufs
        jbd_assert!(commit_tx.iobuf_list.0.len() == commit_tx.shadow_list.0.len());

        while !commit_tx.iobuf_list.0.is_empty() {
            let jb_rc = &commit_tx.iobuf_list.0[0].clone();
            let mut jb = jb_rc.as_ref().borrow_mut();
            Transaction::unfile_buffer(jb_rc, &mut jb, &mut commit_tx);
            let jb_rc = &commit_tx.shadow_list.0[0].clone();
            let mut jb = jb_rc.as_ref().borrow_mut();
            Transaction::unfile_buffer(jb_rc, &mut jb, &mut commit_tx);
            Transaction::file_buffer(
                &commit_tx_rc,
                &mut commit_tx,
                jb_rc,
                &mut jb,
                BufferListType::Forget,
            )?;
        }

        jbd_assert!(commit_tx.iobuf_list.0.is_empty());
        jbd_assert!(commit_tx.shadow_list.0.is_empty());

        // Commit phase 6

        jbd_assert!(commit_tx.state == TransactionState::Commit);
        commit_tx.state = TransactionState::CommitRecord;
        self.write_commit_record(&mut commit_tx)?;

        // Finally, we can do checkpoint
        // processing: any buffers committed as a result of this
        // transaction can be removed from any checkpoint list it was on
        // before.

        // Commit phase 7

        jbd_assert!(commit_tx.sync_datalist.0.is_empty());
        jbd_assert!(commit_tx.buffers.0.is_empty());
        jbd_assert!(commit_tx.checkpoint_list.0.is_empty());
        jbd_assert!(commit_tx.iobuf_list.0.is_empty());
        jbd_assert!(commit_tx.shadow_list.0.is_empty());
        jbd_assert!(commit_tx.log_list.0.is_empty());

        let forget_list = commit_tx.forget.0.clone();

        for jb_rc in forget_list.into_iter() {
            let mut jb = jb_rc.as_ref().borrow_mut();
            if jb.commited_data.is_some() {
                jb.commited_data = None;
                if jb.frozen_data.is_some() {
                    jb.commited_data = jb.frozen_data.clone();
                    jb.frozen_data = None;
                }
            } else if jb.frozen_data.is_some() {
                jb.frozen_data = None;
            }

            if let Some(cp_tx_rc) = &jb.cp_transaction {
                // Remove from checkpoint
                if Rc::ptr_eq(cp_tx_rc, &commit_tx_rc) {
                    commit_tx
                        .checkpoint_list
                        .0
                        .retain(|x| !Rc::ptr_eq(x, &jb_rc));
                } else {
                    let mut tx = cp_tx_rc.as_ref().borrow_mut();
                    tx.checkpoint_list.0.retain(|x| !Rc::ptr_eq(x, &jb_rc));
                }

                jb.cp_transaction = None;
            }

            jbd_assert!(jb.next_transaction.is_none());
            let buf = &jb.buf;

            if buf.jbd_dirty() {
                jb.cp_transaction = Some(commit_tx_rc.clone());
                commit_tx.checkpoint_list.0.push(jb_rc.clone());
            } else {
                jbd_assert!(!buf.dirty());
            }

            Transaction::refile_buffer(&jb_rc, &mut jb, &mut commit_tx);
        }

        jbd_assert!(commit_tx.forget.0.is_empty());

        // Commit phase 8
        commit_tx.state = TransactionState::Finished;

        self.commit_sequence = commit_tx.tid;
        self.committing_transaction = None;

        if !commit_tx.checkpoint_list.0.is_empty() {
            self.checkpoint_transactions.push(commit_tx_rc.clone());
        }

        log::debug!("Commit {} completed.", self.commit_sequence);

        Ok(())
    }

    fn write_commit_record(&mut self, commit_tx: &mut Transaction) -> JBDResult {
        if self.flags.contains(JournalFlag::ABORT) {
            return Ok(());
        }

        let descriptor_rc = self.get_descriptor_buffer()?;
        let descriptor = descriptor_rc.as_ref().borrow_mut();
        let header: &mut Header = descriptor.buf.convert_mut();
        header.magic = JFS_MAGIC_NUMBER.to_be();
        header.block_type = BlockType::CommitBlock.to_u32_be();
        header.sequence = (commit_tx.tid as u32).to_be();

        descriptor.buf.mark_dirty();
        descriptor.buf.sync();

        Ok(())
    }

    fn write_metadata_buffer(
        &mut self,
        tx_rc: &Rc<RefCell<Transaction>>,
        tx: &mut Transaction,
        jb_rc: &Rc<RefCell<JournalBuffer>>,
        jb: &mut JournalBuffer,
        blocknr: u32,
    ) -> JBDResult<(Rc<RefCell<JournalBuffer>>, bool, bool)> {
        let mut need_copy_out = false;
        let mut done_copy_out = false;
        let mut do_escape = false;

        let buf = self.get_buffer_translated(blocknr)?;

        let new_jb_rc = JournalBuffer::new_or_get(&buf);

        let data = if let Some(frozen_data) = &jb.frozen_data {
            done_copy_out = true;
            &frozen_data[..]
        } else {
            jb.buf.buf()
        };

        // Check for escaping
        if u32::from_be_bytes(data[..4].try_into().unwrap()) == JFS_MAGIC_NUMBER {
            need_copy_out = true;
            do_escape = true;
        }

        #[allow(clippy::slow_vector_initialization)]
        if need_copy_out && !done_copy_out {
            let mut new_data: Vec<u8> = Vec::with_capacity(data.len());
            new_data.resize(data.len(), 0);
            new_data.copy_from_slice(data);

            jb.frozen_data = Some(new_data);
            done_copy_out = true;
        } else {
            // A little hack here
            let new_jb = new_jb_rc.as_ref().borrow_mut();
            new_jb.buf.buf_mut().copy_from_slice(data);
        }

        if do_escape {
            if done_copy_out {
                jb.frozen_data.as_mut().unwrap()[0] = 0;
            } else {
                jb.buf.buf_mut()[0] = 0;
            }
        }

        let mut new_jb = new_jb_rc.as_ref().borrow_mut();
        new_jb.transaction = None;
        new_jb.buf.mark_dirty();

        Transaction::file_buffer(tx_rc, tx, jb_rc, jb, BufferListType::Shadow)?;
        Transaction::file_buffer(tx_rc, tx, &new_jb_rc, &mut new_jb, BufferListType::IO)?;

        Ok((new_jb_rc.clone(), do_escape, done_copy_out))
    }

    fn submit_data_buffers(
        &mut self,
        tx_rc: &Rc<RefCell<Transaction>>,
        tx: &mut Transaction,
    ) -> JBDResult {
        let datalist = tx.sync_datalist.0.clone();
        tx.sync_datalist.0.clear();

        for jb_rc in datalist.into_iter() {
            let mut jb = jb_rc.as_ref().borrow_mut();
            let buf = &jb.buf;

            jbd_assert!(buf.jbd_managed());

            if buf.test_clear_dirty() {
                buf.sync();
                Transaction::file_buffer(tx_rc, tx, &jb_rc, &mut jb, BufferListType::Locked)?;
            } else {
                Transaction::unfile_buffer(&jb_rc, &mut jb, tx);
            }
        }

        Ok(())
    }

    pub(crate) fn get_descriptor_buffer(&mut self) -> JBDResult<Rc<RefCell<JournalBuffer>>> {
        let blocknr = self.next_log_block();
        let buf = self.get_buffer_translated(blocknr)?;
        buf.buf_mut().fill(0);

        Ok(JournalBuffer::new_or_get(&buf))
    }

    fn next_log_block(&mut self) -> u32 {
        jbd_assert!(self.free > 1);
        let block = self.head;
        self.head += 1;
        self.free -= 1;
        if self.head == self.last {
            self.head = self.first;
        }
        block
    }
}
