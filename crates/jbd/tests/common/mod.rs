pub mod mock;
pub mod sal;

use std::{cell::RefCell, rc::Rc};

use jbd::{self, err::JBDResult, Handle, Journal};
use sal::UserSystem;

pub const JOURNAL_SIZE: usize = 1024;

pub fn existing_journal(system: Rc<UserSystem>) -> Rc<RefCell<Journal>> {
    let dev = system.block_device();
    let journal = Journal::init_dev(
        system.clone(),
        dev.clone(),
        dev.clone(),
        0,
        JOURNAL_SIZE as u32,
    )
    .unwrap();
    Rc::new(RefCell::new(journal))
}

pub fn create_journal() -> JBDResult<(Rc<UserSystem>, Rc<RefCell<Journal>>)> {
    const NBLOCKS: usize = 2048;
    let system = Rc::new(UserSystem::new("target/test.img", NBLOCKS).unwrap());
    let dev = system.block_device();
    let mut journal = Journal::init_dev(
        system.clone(),
        dev.clone(),
        dev.clone(),
        0,
        JOURNAL_SIZE as u32,
    )
    .unwrap();
    journal.create()?;
    Ok((system, Rc::new(RefCell::new(journal))))
}

pub fn create_handle(journal: Rc<RefCell<Journal>>) -> JBDResult<Rc<RefCell<Handle>>> {
    Journal::start(journal, 128)
}

pub fn setup_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}
