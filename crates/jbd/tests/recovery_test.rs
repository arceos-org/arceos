mod common;
use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use common::{
    create_handle, create_journal, existing_journal, mock::write_random_block, sal::UserSystem,
    setup_logger, JOURNAL_SIZE,
};
use jbd::Journal;

use crate::common::{mock::convert_buf, sal::dev::BLOCK_SIZE};

#[test]
fn test_recovery() {
    setup_logger();
    let (system, journal) = create_journal().unwrap();

    let blocks_dist = Vec::from([[0, 1], [2, 3], [4, 5]]);
    let mut original_data = Vec::new();

    let tx1_data = do_one_transaction(&system, journal.clone(), Vec::from(blocks_dist[0]));
    let tx2_data = do_one_transaction(&system, journal.clone(), Vec::from(blocks_dist[1]));
    let tx3_data = do_one_transaction(&system, journal.clone(), Vec::from(blocks_dist[2]));

    original_data.push(tx1_data);
    original_data.push(tx2_data);
    original_data.push(tx3_data);

    // Recreate the journal without checkpointing the old one.
    let journal = existing_journal(system.clone());
    journal.as_ref().borrow_mut().load().unwrap();

    // The data should have been written to the disk now.
    for i in 0..blocks_dist.len() {
        let blocks = blocks_dist[i];
        for j in 0..blocks.len() {
            let block_id = blocks[j] + JOURNAL_SIZE;
            let mut disk_data = vec![0_u8; BLOCK_SIZE];
            system
                .block_device()
                .read_block(block_id, &mut disk_data[..]);
            assert_eq!(disk_data, original_data[i][j], "block_id: {}", block_id);
        }
    }
}

fn do_one_transaction(
    system: &Rc<UserSystem>,
    journal: Rc<RefCell<Journal>>,
    blocks: Vec<usize>,
) -> Vec<Vec<u8>> {
    let handle_rc = create_handle(journal.clone()).unwrap();
    let mut handle = handle_rc.as_ref().borrow_mut();

    let mut ret = Vec::new();
    for block_id_offset in blocks.iter() {
        let block_id = JOURNAL_SIZE + *block_id_offset;
        // Write a random block.
        let meta_buf = write_random_block(&system, system.block_device().borrow(), block_id);
        let mut data = vec![0_u8; BLOCK_SIZE];
        data.copy_from_slice(convert_buf(&meta_buf));

        let mut disk_data = vec![0_u8; BLOCK_SIZE];
        system
            .block_device()
            .read_block(block_id, &mut disk_data[..]);
        assert!(disk_data != data);

        handle.get_write_access(&meta_buf).unwrap();
        handle.dirty_metadata(&meta_buf).unwrap();

        ret.push(data);
    }

    handle.stop().unwrap();
    journal.borrow_mut().commit_transaction().unwrap();

    ret
}
