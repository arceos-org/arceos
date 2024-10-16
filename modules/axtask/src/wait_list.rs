use alloc::sync::Arc;
use linked_list::{def_node, List};

use crate::AxTaskRef;

def_node!(WaitTaskNode, AxTaskRef);

type Node = Arc<WaitTaskNode>;

pub struct WaitTaskList {
    list: List<Node>,
}

impl WaitTaskList {
    /// Creates a new empty [WaitList].
    pub const fn new() -> Self {
        Self { list: List::new() }
    }

    /// add wait to list back
    pub fn push_back(&mut self, node: Node) {
        self.list.push_back(node);
    }

    /// pop front node from list
    pub fn pop_front(&mut self) -> Option<Node> {
        self.list.pop_front()
    }

    /// remove special task from list
    pub fn remove_task(&mut self, task: &AxTaskRef) -> Option<Node> {
        let mut cursor = self.list.cursor_front_mut();
        let find = loop {
            match cursor.current() {
                Some(node) => {
                    if Arc::ptr_eq(node.inner(), task) {
                        break cursor.remove_current();
                    }
                }
                None => break None,
            }
            cursor.move_next();
        };
        find
    }

    /// Removes the given Node
    ///
    /// # Safety
    ///
    /// Callers must ensure that `data` is either on this list or in no list. It being on another
    /// list leads to memory unsafety.
    pub fn remove(&mut self, node: &Node) -> Option<Node> {
        unsafe { self.list.remove(node) }
    }
}
