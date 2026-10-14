use super::visualization::transform;
use lib::{buffer, tea};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Message {}

pub(crate) struct Model {
    pub ts: buffer::Timestamp,
}

impl tea::Model<Message> for Model {
    fn init() -> (Self, tea::Effects<Message>) {
        (Self {}, Default::default())
    }

    fn update(&mut self, _message: Message) -> tea::Effects<Message> {
        Default::default()
    }
}

pub(crate) struct Completion {
    pub(crate) transform_tree: transform::TransformTreeCompletion,
}

impl Completion {
    pub fn transform_subscription(
        &mut self,
        timeline: &buffer::Timeline<Completion, transform::TransformFrameRecord>,
        ts: buffer::Timestamp,
    ) {
        timeline.query_with(self, ts, |_, _| true, |c, _| c.transform_tree.is_complete());
    }
}

impl buffer::Record<Completion> for transform::TransformFrameRecord {
    fn accum(&self, completion: &mut Completion) {
        self.accum(&mut completion.transform_tree);
    }
}
