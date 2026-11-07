use super::effect::Effects;

pub trait Model<Message>: Sized {
    type Ext;

    fn init() -> (Self, Effects<Message, Self::Ext>);
    fn update(&mut self, message: Message) -> Effects<Message, Self::Ext>;
}
