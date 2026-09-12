use std::{
    pin::Pin,
    rc::{Rc, Weak},
};

pub trait Emitter<Message> {
    fn emit(&self, message: Message);
}

pub type Task = Pin<Box<dyn Future<Output = ()> + 'static>>;
pub trait Spawner {
    fn spawn(&self, task: Task);
}

#[derive(Clone)]
pub struct Capabilities<Message> {
    emitter: Rc<dyn Emitter<Message>>,
    spawner: Rc<dyn Spawner>,
}

impl<Message> Capabilities<Message> {
    pub fn new(emitter: Rc<dyn Emitter<Message>>, spawner: Rc<dyn Spawner>) -> Self {
        Self { emitter, spawner }
    }

    pub fn emitter(&self) -> Weak<dyn Emitter<Message> + 'static> {
        Rc::downgrade(&self.emitter)
    }

    pub fn spawner(&self) -> Weak<dyn Spawner + 'static> {
        Rc::downgrade(&self.spawner)
    }

    pub fn emit(&self, message: Message) {
        self.emitter.emit(message)
    }

    pub fn spawn(&self, task: Task) {
        self.spawner.spawn(task)
    }
}
