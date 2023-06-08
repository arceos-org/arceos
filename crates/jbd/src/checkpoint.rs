use crate::{jbd_assert, journal::JournalFlag, Journal};

impl Journal {
    /// Checkpoint all commits in the log.
    pub fn do_all_checkpoints(&mut self) -> usize {
        let mut count = 0;
        while self.log_do_checkpoint() {
            count += 1;
        }
        count
    }

    /// Checkpoint one commit in the log.
    pub fn log_do_checkpoint(&mut self) -> bool {
        log::debug!("Start checkpoint.");

        self.cleanup_tail();

        // Start writing disk blocks
        if self.checkpoint_transactions.is_empty() {
            log::debug!("No checkpoint transactions.");
            return false;
        }

        let tx_rc = self.checkpoint_transactions[0].clone();
        let mut tx = tx_rc.borrow_mut();
        let count = tx.checkpoint_list.0.len();

        while !tx.checkpoint_list.0.is_empty() {
            let jb_rc = tx.checkpoint_list.0[0].clone();
            let jb = jb_rc.borrow();
            tx.checkpoint_list.0.remove(0);
            jb.buf.sync();
            jb.buf.clear_journal_buffer();
        }

        self.checkpoint_transactions.remove(0);

        log::debug!("Checkpoint done, synced {} buffers.", count);

        true
    }

    fn cleanup_tail(&mut self) {
        let (first_tid, blocknr) = if !self.checkpoint_transactions.is_empty() {
            let tx_rc = &self.checkpoint_transactions[0];
            let tx = tx_rc.borrow();
            (tx.tid, tx.log_start)
        } else if let Some(tx_rc) = &self.committing_transaction {
            let tx = tx_rc.borrow();
            (tx.tid, tx.log_start)
        } else if let Some(tx_rc) = &self.running_transaction {
            let tx = tx_rc.borrow();
            (tx.tid, self.head)
        } else {
            (self.transaction_sequence, self.head)
        };

        jbd_assert!(blocknr != 0);

        if self.tail_sequence == first_tid {
            return;
        }

        jbd_assert!(first_tid > self.tail_sequence);

        let mut freed = blocknr - self.tail;
        if blocknr < self.tail {
            freed += self.last - self.first;
        }

        self.free += freed;
        self.tail_sequence = first_tid;
        self.tail = blocknr;

        if !self.flags.contains(JournalFlag::ABORT) {
            self.update_superblock();
        }
    }
}
