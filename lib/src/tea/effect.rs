use super::platform;

use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use anyhow::Context;

pub type Effects<Message> = Vec<Effect<Message>>;

// FOR LATER: don't like Box<dyn> here
pub type Effect<Message> = Box<dyn FnOnce(Dispatch<Message>)>;

pub fn from_future<Message, F: Future<Output = Message> + 'static>(future: F) -> Effect<Message> {
    Box::new(move |dispatch| {
        platform::spawn_local(async move {
            let msg = future.await;
            let _ = dispatch.call(msg);
        })
    })
}

// FOR LATER: don't like Clone impl here
pub struct Dispatch<Message> {
    queue: Weak<RefCell<VecDeque<Message>>>,
}
impl<Message> Clone for Dispatch<Message> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}
impl<Message> Dispatch<Message> {
    pub fn new(queue: &Rc<RefCell<VecDeque<Message>>>) -> Self {
        Self {
            queue: Rc::downgrade(queue),
        }
    }

    pub fn call(&self, message: Message) -> anyhow::Result<()> {
        let queue = self.queue.upgrade().context("queue expired")?;
        let mut queue = queue.try_borrow_mut()?;
        queue.push_back(message);
        Ok(())
    }
}
