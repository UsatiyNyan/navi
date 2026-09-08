use super::{effect, input, model};

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

    pub fn input(&self) -> input::Input<Message> {
        input::Input::new(self.dispatch.clone())
    }

    pub fn new() -> Self {
        let (model, effects_init) = Model::init();
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
        let mut queue = self.queue.try_borrow_mut().ok()?;
        queue.pop_front()
    }

    fn start_effects(&self, effects_batch: effect::Effects<Message>) {
        for effect in effects_batch {
            (effect)(self.dispatch.clone());
        }
    }
}
