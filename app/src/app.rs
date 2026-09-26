use lib::{buffer, render, tea};

#[derive(Debug)]
pub(crate) enum Message {}

pub(crate) struct Model {}

impl tea::Model<Message> for Model {
    fn init() -> (Self, tea::Effects<Message>) {
        (Self {}, Default::default())
    }

    fn update(&mut self, _message: Message) -> tea::Effects<Message> {
        Default::default()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Record {}

impl buffer::Record<Record> for Record {
    fn merge(&mut self, _other: Record) {}

    fn combine(&self, _other: &Record) -> Record {
        _other.clone()
    }
}
