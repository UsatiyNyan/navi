use crate::tea::Capabilities;

use super::{capabilities, effect, model, queue};

use std::rc::Rc;

pub struct App<Model, Message> {
    model: Model,
    queue: Rc<queue::Queue<Message>>,
    capabilities: capabilities::Capabilities<Message>,
}

pub struct AppSettings {
    spawner: Rc<dyn capabilities::Spawner>,
}

impl<Model, Message: 'static> App<Model, Message>
where
    Model: model::Model<Message>,
{
    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn capabilities(&self) -> &Capabilities<Message> {
        &self.capabilities
    }

    pub fn new(settings: AppSettings) -> Self {
        let (model, effects_init) = Model::init();
        let queue = Rc::new(queue::Queue::new());
        let capabilities = capabilities::Capabilities::new(queue.clone(), settings.spawner);
        let a_self = Self {
            model,
            queue,
            capabilities,
        };

        a_self.start_effects(effects_init);

        a_self
    }

    pub fn run_once(&mut self) -> Option<&Model> {
        let mut effects_batch: effect::Effects<Message> = Default::default();
        let mut is_changed = false;

        while let Some(message) = self.drain_once() {
            let effects_once = self.model.update(message);
            effects_batch.extend(effects_once);
            is_changed = true;
        }

        self.start_effects(effects_batch);

        if is_changed { Some(&self.model) } else { None }
    }

    fn drain_once(&mut self) -> Option<Message> {
        self.queue.pop()
    }

    fn start_effects(&self, effects_batch: effect::Effects<Message>) {
        for effect in effects_batch {
            (effect)(&self.capabilities);
        }
    }
}
