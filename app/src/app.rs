use lib::{buffer, render, tea};

#[derive(Debug)]
pub(crate) enum Message {}

pub(crate) struct Model {}

impl tea::Model<Message> for Model {
    fn init() -> (Self, tea::Effects<Message>) {
        (Self {}, Default::default())
    }

    fn update(&mut self, message: Message) -> tea::Effects<Message> {
        todo!();
        Default::default()
    }
}

pub fn render(model: &Model, render_handle: &mut render::Handle, buffer_state: &mut buffer::State) {
    todo!()
}
