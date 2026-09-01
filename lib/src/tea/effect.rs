use std::{cell::RefCell, collections::VecDeque, rc::Weak};

pub type Effects<Message> = Vec<Effect<Message>>;

// FOR LATER: don't like Box<dyn> here
pub type Effect<Message> = Box<dyn FnOnce(Dispatch<Message>)>;

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
    pub fn call(&self, message: Message) {
        if let Some(queue) = self.queue.upgrade() {
            if let Ok(mut queue) = queue.try_borrow_mut() {
                queue.push_back(message);
            }
        }
    }
}
