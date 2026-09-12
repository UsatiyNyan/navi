use super::capabilities;

use std::{cell::RefCell, collections::VecDeque};

pub(crate) struct Queue<Message> {
    inner: RefCell<VecDeque<Message>>,
}

impl<Message> Queue<Message> {
    pub(crate) fn pop(&self) -> Option<Message> {
        self.inner
            .try_borrow_mut()
            .ok()
            .and_then(|mut q| q.pop_back())
    }
}

impl<Message> Queue<Message> {
    pub(crate) fn new() -> Self {
        Self {
            inner: RefCell::new(Default::default()),
        }
    }
}

impl<Message> capabilities::Emitter<Message> for Queue<Message> {
    fn emit(&self, message: Message) {
        self.inner.borrow_mut().push_back(message)
    }
}
