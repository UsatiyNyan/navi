use lib::{buffer, tea};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Message {
    Tick { dt: buffer::Duration },
    Scrub { cursor: buffer::Timestamp },
    SetPlayback(Playback),
    Ingested { ts: buffer::Timestamp }, // other metadata
}

pub(crate) struct Model {
    pub playback: Playback,
    pub cursor: Option<buffer::Timestamp>,
    pub oldest: Option<buffer::Timestamp>,
    pub newest: Option<buffer::Timestamp>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Playback {
    Live,
    Playing { speed: f64 },
    Paused,
}

impl tea::Model<Message> for Model {
    fn init() -> (Self, tea::Effects<Message>) {
        (
            Self {
                playback: Playback::Paused,
                newest: None,
                cursor: None,
                oldest: None,
            },
            Default::default(),
        )
    }

    fn update(&mut self, message: Message) -> tea::Effects<Message> {
        match message {
            Message::Tick { dt } => {
                if let (Some(cursor), Playback::Playing { speed }) = (self.cursor, self.playback) {
                    self.cursor = Some(cursor + dt.mul_f64(speed));
                }
            }
            Message::Scrub { cursor } => {
                self.cursor = Some(cursor);
                self.playback = Playback::Paused;
            }
            Message::SetPlayback(playback) => self.playback = playback,
            Message::Ingested { ts } => {
                self.oldest = Some(self.oldest.unwrap_or(ts).min(ts));
                self.newest = Some(self.newest.unwrap_or(ts).max(ts));
                if let Playback::Live = self.playback {
                    self.cursor = self.newest;
                }
            }
        }
        Default::default()
    }
}
