extern crate alloc;

use core::cell::RefCell;

use alloc::{collections::BTreeMap, rc::Rc, vec::Vec};
use bitflags::bitflags;

use crate::{
    config::{JFS_MAGIC_NUMBER, JFS_MIN_JOURNAL_BLOCKS, MIN_LOG_RESERVED_BLOCKS},
    disk::{BlockType, Superblock},
    err::{JBDError, JBDResult},
    jbd_assert,
    revoke::RevokeRecord,
    sal::{BlockDevice, Buffer, System},
    tx::{Handle, Tid, Transaction, TransactionState},
};

#[cfg(feature = "debug")]
use crate::disk::Display;

pub struct Journal {
    pub(crate) system: Rc<dyn System>,
    pub(crate) sb_buffer: Rc<dyn Buffer>,
    pub(crate) format_version: i32,
    pub(crate) flags: JournalFlag,
    pub(crate) errno: i32, // TODO: Strongly-typed error?
    pub(crate) running_transaction: Option<Rc<RefCell<Transaction>>>,
    pub(crate) committing_transaction: Option<Rc<RefCell<Transaction>>>,
    /// Journal head: identifies the first unused block in the journal.
    pub(crate) head: u32,
    /// Journal tail: identifies the oldest still-used block in the journal
    pub(crate) tail: u32,
    /// Journal free: how many free blocks are there in the journal?
    pub(crate) free: u32,
    /// Journal start: the block number of the first usable block in the journal
    pub(crate) first: u32,
    /// Journal end: the block number of the last usable block in the journal
    pub(crate) last: u32,
    /// Sequence number of the oldest transaction in the log
    pub(crate) tail_sequence: Tid,
    /// Sequence number of the next transaction to grant
    pub(crate) transaction_sequence: Tid,
    /// Sequence number of the most recently committed transaction
    pub(crate) commit_sequence: Tid,
    /// Sequence number of the most recent transaction wanting commit
    pub(crate) commit_request: Tid,
    /// List of all transactions waiting for checkpointing
    pub(crate) checkpoint_transactions: Vec<Rc<RefCell<Transaction>>>,
    /// Block devices
    pub(crate) devs: JournalDevs,
    /// Total maximum capacity of the journal region on disk
    pub(crate) maxlen: u32,
    /// Maximum number of metadata buffers to allow in a single compound
    /// commit transaction
    pub(crate) max_transaction_buffers: u32,
    // commit_interval: usize,
    pub(crate) wbuf: Vec<Rc<dyn Buffer>>,
    pub(crate) revoke_tables: [BTreeMap<u32, RevokeRecord>; 2],
    pub(crate) current_revoke_table: usize,
}

bitflags! {
    pub(crate) struct JournalFlag: usize {
        const UNMOUNT = 0x001;
        const ABORT = 0x002;
        const ACK_ERR = 0x004;
        const FLUSHED = 0x008;
        const LOADED = 0x010;
        const BARRIER = 0x020;
        const ABORT_ON_SYNCDATA_ERR = 0x040;
    }
}

pub(crate) struct JournalDevs {
    pub(crate) dev: Rc<dyn BlockDevice>,
    pub(crate) blk_offset: u32,
    #[allow(unused)]
    pub(crate) fs_dev: Rc<dyn BlockDevice>,
}

/// Public interfaces.
impl Journal {
    /// Initialize an in-memory journal structure with a block device.
    pub fn init_dev(
        system: Rc<dyn System>,
        dev: Rc<dyn BlockDevice>,
        fs_dev: Rc<dyn BlockDevice>,
        start: u32,
        len: u32,
    ) -> JBDResult<Self> {
        let devs = JournalDevs {
            dev,
            blk_offset: start,
            fs_dev,
        };
        let sb_buffer = system
            .get_buffer_provider()
            .get_buffer(&devs.dev, devs.blk_offset as usize);
        if sb_buffer.is_none() {
            return Err(JBDError::IOError);
        }

        let ret = Self {
            system: system.clone(),
            sb_buffer: sb_buffer.unwrap(),
            format_version: 0,
            flags: JournalFlag::ABORT,
            errno: 0,
            running_transaction: None,
            committing_transaction: None,
            head: 0,
            tail: 0,
            free: 0,
            first: 0,
            last: 0,
            tail_sequence: 0,
            transaction_sequence: 0,
            commit_sequence: 0,
            commit_request: 0,
            checkpoint_transactions: Vec::new(),
            devs,
            maxlen: len,
            max_transaction_buffers: 0,
            wbuf: Vec::new(),
            revoke_tables: [BTreeMap::new(), BTreeMap::new()],
            current_revoke_table: 0,
        };

        Ok(ret)
    }

    pub fn create(&mut self) -> JBDResult {
        if self.maxlen < JFS_MIN_JOURNAL_BLOCKS {
            log::error!("Journal too small: {} blocks.", self.maxlen);
            return Err(JBDError::InvalidJournalSize);
        }

        log::debug!("Zeroing out journal blocks.");
        for i in 0..self.maxlen {
            let block_id = i;
            let page_head = self.get_buffer_translated(block_id)?;
            let buf = page_head.buf_mut();
            buf.fill(0);
            page_head.sync();
        }

        log::debug!("Journal cleared.");

        let sb = self.superblock_mut();

        sb.header.magic = JFS_MAGIC_NUMBER.to_be();
        sb.header.block_type = BlockType::SuperblockV2.to_u32_be();

        sb.block_size = (self.devs.dev.block_size() as u32).to_be();
        sb.maxlen = self.maxlen.to_be();
        sb.first = 1_u32.to_be();

        self.transaction_sequence = 1;

        self.flags.remove(JournalFlag::ABORT);

        self.format_version = 2;

        self.reset()
    }

    pub fn load(&mut self) -> JBDResult {
        self.load_superblock()?;

        self.recover()?;
        self.reset()?;

        self.flags.remove(JournalFlag::ABORT);
        self.flags.insert(JournalFlag::LOADED);

        Ok(())
    }

    pub fn start(journal_rc: Rc<RefCell<Journal>>, nblocks: u32) -> JBDResult<Rc<RefCell<Handle>>> {
        let journal = journal_rc.as_ref().borrow();
        if let Some(current_handle) = journal.system.get_current_handle() {
            return Ok(current_handle);
        }
        let mut handle = Handle::new(nblocks);
        drop(journal);
        start_handle(&journal_rc, &mut handle)?;
        let handle = Rc::new(RefCell::new(handle));
        journal_rc
            .as_ref()
            .borrow()
            .system
            .set_current_handle(Some(handle.clone()));
        Ok(handle)
    }

    pub fn destroy(&mut self) -> JBDResult {
        if self.running_transaction.is_some() {
            self.commit_transaction()?;
        }

        self.do_all_checkpoints();

        jbd_assert!(self.running_transaction.is_none());
        jbd_assert!(self.committing_transaction.is_none());
        jbd_assert!(self.checkpoint_transactions.is_empty());

        if !self.flags.contains(JournalFlag::ABORT) {
            self.tail = 0;
            self.tail_sequence = self.transaction_sequence + 1;
            self.update_superblock();
        } else {
            return Err(JBDError::JournalAborted);
        }

        Ok(())
    }
}

/// Internal helper functions.
impl Journal {
    /// Given a journal_t structure, initialize the various fields for
    /// startup of a new journaling session.  We use this both when creating
    /// a journal, and after recovering an old journal to reset it for
    /// subsequent use.
    fn reset(&mut self) -> JBDResult {
        let sb = self.superblock_mut();

        let first = u32::from_be(sb.first);
        let last = u32::from_be(sb.maxlen);

        if first + JFS_MIN_JOURNAL_BLOCKS > last + 1 {
            log::error!("Journal too small: blocks {}-{}.", first, last);
            // TODO: Discard
            return Err(JBDError::InvalidJournalSize);
        }

        self.first = first;
        self.last = last;

        self.head = first;
        self.tail = first;
        self.free = last - first;

        self.tail_sequence = self.transaction_sequence;
        self.commit_sequence = self.transaction_sequence - 1;
        self.commit_request = self.commit_sequence;

        self.max_transaction_buffers = self.maxlen / 4;

        self.update_superblock();

        Ok(())
    }

    /// Load the on-disk journal superblock and read the key fields.
    fn load_superblock(&mut self) -> JBDResult {
        self.validate_superblock()?;

        let sb = self.superblock_ref();

        let tail_sequence = u32::from_be(sb.sequence) as u16;
        let tail = u32::from_be(sb.start);
        let first = u32::from_be(sb.first);
        let last = u32::from_be(sb.maxlen);
        let errno = i32::from_be(sb.errno);

        #[cfg(feature = "debug")]
        log::debug!("Loaded superblock: {}", sb.display(0));

        self.tail_sequence = tail_sequence;
        self.tail = tail;
        self.first = first;
        self.last = last;
        self.errno = errno;

        Ok(())
    }

    /// Update a journal's dynamic superblock fields and write it to disk.
    pub(crate) fn update_superblock(&mut self) {
        let sb = self.superblock_mut();

        if sb.start == 0 && self.tail_sequence == self.transaction_sequence {
            log::debug!("Skipping superblock update on newly created / recovered journal.");
            self.flags.insert(JournalFlag::FLUSHED);
            return;
        }

        #[cfg(not(feature = "debug"))]
        log::debug!("Updating superblock.");
        sb.sequence = (self.tail_sequence as u32).to_be();
        sb.start = self.tail.to_be();
        sb.errno = self.errno.to_be();

        self.sb_buffer.sync();

        #[cfg(feature = "debug")]
        log::debug!("Updating superblock: {}", sb.display(0));

        if self.tail != 0 {
            self.flags.remove(JournalFlag::FLUSHED);
        } else {
            self.flags.insert(JournalFlag::FLUSHED);
        }
    }

    fn validate_superblock(&mut self) -> JBDResult {
        // No need to test buffer_uptodate here as in our implementation as far,
        // the buffer will always be valid.
        let sb = self.superblock_ref();

        if sb.header.magic != JFS_MAGIC_NUMBER.to_be()
            || sb.block_size != (self.devs.dev.block_size() as u32).to_be()
        {
            log::error!("Invalid journal superblock magic number or block size.");
            return Err(JBDError::InvalidSuperblock);
        }

        let block_type = BlockType::from_u32_be(sb.header.block_type)?;

        match block_type {
            BlockType::SuperblockV1 => self.format_version = 1,
            BlockType::SuperblockV2 => self.format_version = 2,
            _ => {
                log::error!("Invalid journal superblock block type.");
                return Err(JBDError::InvalidSuperblock);
            }
        }

        if u32::from_be(self.superblock_ref().maxlen) <= self.maxlen {
            self.maxlen = u32::from_be(self.superblock_ref().maxlen);
        } else {
            log::error!("Journal too short.");
            // Linux returns -EINVAL here, so as we.
            return Err(JBDError::InvalidSuperblock);
        }

        if u32::from_be(self.superblock_ref().first) == 0
            || u32::from_be(self.superblock_ref().first) >= self.maxlen
        {
            log::error!("Journal has invalid start block.");
            return Err(JBDError::InvalidSuperblock);
        }

        Ok(())
    }

    pub(crate) fn superblock_ref(&self) -> &Superblock {
        self.sb_buffer.convert::<Superblock>()
    }

    pub(crate) fn superblock_mut(&self) -> &mut Superblock {
        self.sb_buffer.convert_mut::<Superblock>()
    }

    pub(crate) fn get_buffer_translated(&self, block_id: u32) -> JBDResult<Rc<dyn Buffer>> {
        self.system
            .get_buffer_provider()
            .get_buffer(&self.devs.dev, (block_id + self.devs.blk_offset) as usize)
            .map_or(Err(JBDError::IOError), |bh| Ok(bh))
    }

    pub(crate) fn get_buffer_direct(&self, block_id: u32) -> JBDResult<Rc<dyn Buffer>> {
        self.system
            .get_buffer_provider()
            .get_buffer(&self.devs.dev, (block_id) as usize)
            .map_or(Err(JBDError::IOError), |bh| Ok(bh))
    }

    /// Start a new transaction in the journal, equivalent to get_transaction()
    /// in linux.
    fn set_transaction(&mut self, tx: &Rc<RefCell<Transaction>>) {
        {
            let mut tx_mut = tx.as_ref().borrow_mut();
            tx_mut.state = TransactionState::Running;
            tx_mut.start_time = self.system.get_time();
            tx_mut.tid = self.transaction_sequence;
        }
        self.transaction_sequence += 1;
        self.running_transaction = Some(tx.clone());
    }

    pub(crate) fn log_space_left(&self) -> u32 {
        let mut left = self.free as i32;
        left -= MIN_LOG_RESERVED_BLOCKS as i32;
        if left <= 0 {
            0
        } else {
            (left - (left >> 3)) as u32
        }
    }
}

pub(crate) fn start_handle(journal_rc: &Rc<RefCell<Journal>>, handle: &mut Handle) -> JBDResult {
    let mut journal = journal_rc.as_ref().borrow_mut();
    let nblocks = handle.buffer_credits;

    if nblocks > journal.max_transaction_buffers {
        log::error!(
            "Transaction requires too many credits ({} > {}).",
            nblocks,
            journal.max_transaction_buffers
        );
        return Err(JBDError::NotEnoughSpace);
    }

    log::debug!("New handle going live.");

    if journal.flags.contains(JournalFlag::ABORT)
        || (journal.errno != 0 && !journal.flags.contains(JournalFlag::ACK_ERR))
    {
        log::error!("Journal has aborted.");
        return Err(JBDError::IOError);
    }

    if journal.running_transaction.is_none() {
        let tx = Transaction::new(Rc::downgrade(journal_rc));
        let tx = Rc::new(RefCell::new(tx));
        journal.set_transaction(&tx);
    }

    let transaction_rc = journal.running_transaction.as_ref().unwrap().clone();
    let mut transaction = journal
        .running_transaction
        .as_ref()
        .unwrap()
        .as_ref()
        .borrow_mut();

    if transaction.state == TransactionState::Locked {
        todo!("Wait for transaction to unlock.");
    }

    let needed = transaction.outstanding_credits + nblocks;

    if needed > journal.max_transaction_buffers {
        todo!("Wait for previous transaction to commit.");
    }

    if journal.log_space_left() < needed {
        todo!("Wait for checkpoint.");
    }

    handle.transaction = Some(transaction_rc);
    transaction.outstanding_credits += nblocks;
    transaction.updates += 1;
    transaction.handle_count += 1;

    log::debug!(
        "Handle now has {} credits.",
        transaction.outstanding_credits
    );

    Ok(())
}
