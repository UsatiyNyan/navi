use lib::{buffer, tea};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Message {}

pub(crate) struct Model {
    pub ts: buffer::Timestamp,
}

impl tea::Model<Message> for Model {
    fn init() -> (Self, tea::Effects<Message>) {
        (
            Self {
                ts: buffer::Timestamp::now(),
            },
            Default::default(),
        )
    }

    fn update(&mut self, _message: Message) -> tea::Effects<Message> {
        Default::default()
    }
}
