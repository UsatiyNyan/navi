use super::effect::Dispatch;

pub struct Input<Message>(Dispatch<Message>);

impl<Message> Input<Message> {
    pub fn new(dispatch: Dispatch<Message>) -> Self {
        Self(dispatch)
    }

    pub fn send(&self, message: Message) -> anyhow::Result<()> {
        self.0.call(message)
    }
}
