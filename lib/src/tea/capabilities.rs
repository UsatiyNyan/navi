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
pub type ExtPtr<Ext> = Rc<Ext>;

#[cfg(not(target_arch = "wasm32"))]
pub type ExtPtr<Ext> = Arc<Ext>;

#[cfg(target_arch = "wasm32")]
pub type Task = Pin<Box<dyn Future<Output = ()> + 'static>>;

#[cfg(not(target_arch = "wasm32"))]
pub type Task = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub trait Spawner {
    fn spawn(&self, task: Task);
}

pub struct Capabilities<Message, Ext> {
    emitter: EmitterPtr<Message>,
    spawner: Rc<dyn Spawner>,
    ext: ExtPtr<Ext>,
}

impl<Message, Ext> Capabilities<Message, Ext> {
    pub fn new(emitter: EmitterPtr<Message>, spawner: Rc<dyn Spawner>, ext: ExtPtr<Ext>) -> Self {
        Self {
            emitter,
            spawner,
            ext,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn emitter(&self) -> rc::Weak<dyn Emitter<Message> + 'static> {
        Rc::downgrade(&self.emitter)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn emitter(&self) -> sync::Weak<dyn Emitter<Message> + Send + Sync + 'static> {
        Arc::downgrade(&self.emitter)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn ext(&self) -> rc::Weak<Ext> {
        Rc::downgrade(&self.ext)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn ext(&self) -> sync::Weak<Ext> {
        Arc::downgrade(&self.ext)
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
