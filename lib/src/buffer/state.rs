use super::record;
use super::timeline;

pub struct State<T: record::Record<T> + Clone> {
    pub timeline: timeline::Timeline<T>,
}

impl<T: record::Record<T> + Clone> State<T> {
    pub fn new(timeline_options: timeline::TimelineOptions) -> Self {
        Self {
            timeline: timeline::Timeline::new(timeline_options),
        }
    }
}
