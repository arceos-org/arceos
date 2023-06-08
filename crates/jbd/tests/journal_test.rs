mod common;
use common::create_journal;

#[test]
fn test_create() {
    create_journal().unwrap();
}
