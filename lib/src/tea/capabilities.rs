#[cfg(not(target_arch = "wasm32"))]
use std::sync::{self, Arc};
use std::{
    pin::Pin,
    rc::{self, Rc},
};

pub trait Emitter<Message> {
    fn emit(&self, message: Message);
}

#[cfg(target_arch = "wasm32")]
pub type EmitterPtr<Message> = Rc<dyn Emitter<Message>>;

#[cfg(not(target_arch = "wasm32"))]
pub type EmitterPtr<Message> = Arc<dyn Emitter<Message> + Send + Sync>;

#[cfg(target_arch = "wasm32")]
pub type Task = Pin<Box<dyn Future<Output = ()> + 'static>>;

#[cfg(not(target_arch = "wasm32"))]
pub type Task = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub trait Spawner {
    fn spawn(&self, task: Task);
}

#[derive(Clone)]
pub struct Capabilities<Message> {
    emitter: EmitterPtr<Message>,
    spawner: Rc<dyn Spawner>,
}

impl<Message> Capabilities<Message> {
    pub fn new(emitter: EmitterPtr<Message>, spawner: Rc<dyn Spawner>) -> Self {
        Self { emitter, spawner }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn emitter(&self) -> rc::Weak<dyn Emitter<Message> + 'static> {
        Rc::downgrade(&self.emitter)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn emitter(&self) -> sync::Weak<dyn Emitter<Message> + Send + Sync + 'static> {
        Arc::downgrade(&self.emitter)
    }

    pub fn spawner(&self) -> rc::Weak<dyn Spawner + 'static> {
        Rc::downgrade(&self.spawner)
    }

    pub fn emit(&self, message: Message) {
        self.emitter.emit(message)
    }

    pub fn spawn(&self, task: Task) {
        self.spawner.spawn(task)
    }
}
