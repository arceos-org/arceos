extern crate alloc;

use core::mem::size_of;

use alloc::rc::Rc;

use crate::{
    config::JFS_MAGIC_NUMBER,
    disk::{BlockTag, BlockType, Header, RevokeBlockHeader, TagFlag},
    err::{JBDError, JBDResult},
    sal::Buffer,
    tx::Tid,
    Journal,
};

pub(crate) struct RecoveryInfo {
    start_transcation: Tid,
    end_transaction: Tid,

    num_replays: usize,
    num_revokes: usize,
    num_revoke_hits: usize,
}

impl RecoveryInfo {
    fn new() -> Self {
        Self {
            start_transcation: 0,
            end_transaction: 0,
            num_replays: 0,
            num_revokes: 0,
            num_revoke_hits: 0,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum PassType {
    Scan,
    Revoke,
    Replay,
}

impl Journal {
    pub(crate) fn recover(&mut self) -> JBDResult {
        let sb = self.superblock_ref();

        // If sb.start == 0, the journal has already been safely unmounted.
        if sb.start == 0 {
            log::debug!(
                "No recovery required, last transaction: {}",
                u32::from_be(sb.sequence)
            );
            self.transaction_sequence = (u32::from_be(sb.sequence) + 1) as u16;
            return Ok(());
        }

        let mut info = RecoveryInfo::new();
        self.do_one_pass(&mut info, PassType::Scan)?;
        self.do_one_pass(&mut info, PassType::Revoke)?;
        log::debug!("Recovery pass 1 complete, {} revokes", info.num_revokes);
        self.do_one_pass(&mut info, PassType::Replay)?;

        log::debug!(
            "Recovery complete, recovered transactions {} to {}",
            info.start_transcation,
            info.end_transaction - 1
        );

        log::debug!(
            "Recovery stats: {} replayed, {} revoked, {} revoke hits",
            info.num_replays,
            info.num_revokes,
            info.num_revoke_hits
        );

        info.end_transaction += 1;
        self.transaction_sequence = info.end_transaction;

        self.clear_revoke();

        Ok(())
    }
}

impl Journal {
    fn do_one_pass(&mut self, info: &mut RecoveryInfo, pass_type: PassType) -> JBDResult {
        let sb = self.superblock_mut();
        let mut next_commit_id = u32::from_be(sb.sequence);
        let mut next_log_block = u32::from_be(sb.start);
        let first_commit_id = next_commit_id;

        if pass_type == PassType::Scan {
            info.start_transcation = first_commit_id as u16;
        }

        log::debug!("Start recovery pass {:?}", pass_type);

        // Now we walk through the log, transaction by transaction,
        // making sure that each transaction has a commit block in the
        // expected place.  Each complete transaction gets replayed back
        // into the main filesystem.

        loop {
            if pass_type != PassType::Scan && next_commit_id >= info.end_transaction as u32 {
                break;
            }
            log::trace!(
                "Scanning sequence {} at {}/{}",
                next_commit_id,
                next_log_block,
                self.last
            );

            let buf = self.read_block(next_log_block)?;

            next_log_block = self.wrap(next_log_block + 1);

            let header: &Header = buf.convert();
            if header.magic != JFS_MAGIC_NUMBER.to_be() {
                // Reach the end of the log
                break;
            }

            let block_type = BlockType::from_u32_be(header.block_type)?;
            let sequence = u32::from_be(header.sequence);
            log::trace!("Found block type {:?}, sequence {}", block_type, sequence);

            if sequence != next_commit_id {
                // Reach the end of the log
                break;
            }

            match block_type {
                BlockType::DescriptorBlock => {
                    if pass_type != PassType::Replay {
                        // Just skip the blocks it describes
                        let num_tags = count_tags(&buf, buf.size());
                        next_log_block = self.wrap(next_log_block + num_tags);
                        log::trace!("Descriptor block has {} tags.", num_tags);
                        continue;
                    }
                    let mut offset = size_of::<Header>();
                    while offset < buf.size() {
                        let tag = buf.convert_offset::<BlockTag>(offset);
                        let flag = TagFlag::from_bits_truncate(u32::from_be(tag.flag));

                        let io_block = next_log_block;
                        next_log_block = self.wrap(next_log_block + 1);
                        let io_buf = self.read_block(io_block)?;

                        let blocknr = u32::from_be(tag.block_nr);
                        if self.test_revoke(blocknr, next_commit_id as Tid) {
                            log::trace!("Replay: skipping revoked block {}", blocknr);
                            info.num_revoke_hits += 1;
                        } else {
                            log::trace!("Replay: replaying block {}", blocknr);
                            // The block is should not be offseted.
                            let new_buf = self.get_buffer_direct(blocknr)?;
                            new_buf.buf_mut().copy_from_slice(io_buf.buf());

                            if flag.contains(TagFlag::ESCAPE) {
                                new_buf.buf_mut()[..4]
                                    .copy_from_slice(&u32::to_be_bytes(JFS_MAGIC_NUMBER));
                            }
                            new_buf.sync();

                            info.num_replays += 1;
                        }
                        log::trace!("flags: {:?}, blocknr: {}", flag, blocknr);

                        offset += size_of::<BlockTag>();
                        if !flag.contains(TagFlag::SAME_UUID) {
                            offset += 16;
                        }
                        if flag.contains(TagFlag::LAST_TAG) {
                            break;
                        }
                    }
                }
                BlockType::CommitBlock => {
                    next_commit_id += 1;
                }
                BlockType::RevokeBlock => {
                    if pass_type != PassType::Revoke {
                        continue;
                    }
                    self.scan_revoke_records(&buf, next_commit_id as Tid, info)?
                }
                _ => {
                    log::error!("Unrecognized block type {:?} in journal", block_type);
                    break;
                }
            }
        }

        if pass_type == PassType::Scan {
            info.end_transaction = next_commit_id as u16;
        } else if info.end_transaction != next_commit_id as u16 {
            log::error!(
                "Recovery pass {:?} ended at transaction {}, expected {}",
                pass_type,
                next_commit_id,
                info.end_transaction
            );
            return Err(JBDError::IOError);
        }

        Ok(())
    }

    fn read_block(&self, offset: u32) -> JBDResult<Rc<dyn Buffer>> {
        if offset >= self.maxlen {
            log::error!("Corrupted journal superblock");
            return Err(JBDError::InvalidSuperblock);
        }

        self.get_buffer_translated(offset)
    }

    fn scan_revoke_records(
        &mut self,
        buf: &Rc<dyn Buffer>,
        sequence: Tid,
        info: &mut RecoveryInfo,
    ) -> JBDResult {
        let header = buf.convert::<RevokeBlockHeader>();
        let mut offset = size_of::<RevokeBlockHeader>();
        let max = u32::from_be(header.count) as usize;

        while offset < max {
            let blocknr = u32::from_be(*buf.convert_offset::<u32>(offset));
            offset += 4;
            self.set_revoke(blocknr, sequence);
            info.num_revokes += 1;
        }

        Ok(())
    }

    fn wrap(&self, block: u32) -> u32 {
        if block >= self.last {
            block - (self.last - self.first)
        } else {
            block
        }
    }
}

fn count_tags(buf: &Rc<dyn Buffer>, size: usize) -> u32 {
    let mut offset = size_of::<Header>();
    let mut num = 0;

    while offset <= size {
        let tag = buf.convert_offset::<BlockTag>(offset);

        num += 1;
        offset += size_of::<BlockTag>();

        let flag = TagFlag::from_bits_truncate(u32::from_be(tag.flag));
        if !flag.contains(TagFlag::SAME_UUID) {
            offset += 16;
        }
        if flag.contains(TagFlag::LAST_TAG) {
            break;
        }
    }

    num
}
