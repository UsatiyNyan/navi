use std::collections::HashMap;

use super::record;
use super::timeline;

pub struct State<T: record::Record> {
    topics: HashMap<String, timeline::Timeline<T>>,
    options: timeline::TimelineOptions,
}

impl<T: record::Record> State<T> {
    pub fn new(timeline_options: timeline::TimelineOptions) -> Self {
        Self {
            topics: Default::default(),
            options: timeline_options,
        }
    }

    pub fn get(&self, topic: &str) -> Option<&timeline::Timeline<T>> {
        self.topics.get(topic)
    }

    pub fn get_mut(&mut self, topic: &str) -> Option<&mut timeline::Timeline<T>> {
        self.topics.get_mut(topic)
    }

    pub fn emplace(&mut self, topic: String) -> &mut timeline::Timeline<T> {
        self.topics
            .entry(topic)
            .or_insert_with(|| timeline::Timeline::new(self.options.clone()))
    }
}
