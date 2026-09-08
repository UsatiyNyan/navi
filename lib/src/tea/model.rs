use super::effect::Effects;

pub trait Model<Message>: Sized {
    fn init() -> (Self, Effects<Message>);
    fn update(&mut self, message: Message) -> Effects<Message>;
}
