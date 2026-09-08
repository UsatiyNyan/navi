use super::effect;
use super::model;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub struct App<Model, Message> {
    model: Model,
    queue: Rc<RefCell<VecDeque<Message>>>,
    dispatch: effect::Dispatch<Message>,
}

impl<Model, Message> App<Model, Message>
where
    Model: model::Model<Message>,
{
    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn new<Init>(init: Init) -> Self
    where
        Init: FnOnce() -> (Model, effect::Effects<Message>),
    {
        let (model, effects_init) = (init)();
        let queue = Rc::new(RefCell::new(VecDeque::<Message>::new()));
        let dispatch = effect::Dispatch::new(&queue);
        let a_self = Self {
            model,
            queue,
            dispatch,
        };

        a_self.start_effects(effects_init);

        a_self
    }

    pub fn run_once(&mut self) -> Option<&Model> {
        let mut is_changed = false;

        let mut effects_batch: effect::Effects<Message> = Default::default();
        while let Some(message) = self.drain_once() {
            let effects_once = self.model.update(message);
            effects_batch.extend(effects_once);
            is_changed = true;
        }

        self.start_effects(effects_batch);

        if is_changed { Some(&self.model) } else { None }
    }

    fn drain_once(&mut self) -> Option<Message> {
        let mut queue = self.queue.try_borrow_mut().ok()?;
        queue.pop_front()
    }

    fn start_effects(&self, effects_batch: effect::Effects<Message>) {
        for effect in effects_batch {
            (effect)(self.dispatch.clone());
        }
    }
}
