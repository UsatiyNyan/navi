use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub struct Runtime<Model, Message, Update>
where
    Update: Fn(Model, Message) -> Model,
{
    model: Model,
    update: Update,
    queue: Rc<RefCell<VecDeque<Message>>>,
}

impl<Model, Message, Update> Runtime<Model, Message, Update> where
    Update: Fn(Model, Message) -> Model
{
}
