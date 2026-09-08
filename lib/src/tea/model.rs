use super::effect::Effects;

pub trait Model<Message> {
    fn update(&mut self, message: Message) -> Effects<Message>;
}
