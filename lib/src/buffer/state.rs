use std::collections::HashMap;

use super::record;
use super::timeline;
use super::topic;

pub struct State<'a> {
    topics: HashMap<String, Box<dyn topic::Topic + 'a>>,
    timeline_options: timeline::TimelineOptions,
}

impl<'a> State<'a> {
    pub fn new(timeline_options: timeline::TimelineOptions) -> Self {
        Self {
            topics: Default::default(),
            timeline_options,
        }
    }

    pub fn subscribe<T, S>(&mut self, topic_path: String, subscription: S)
    where
        S: FnMut(timeline::Timestamp, &timeline::Timeline<T>) + 'a,
        T: record::Record + 'a,
    {
        self.topics.insert(
            topic_path,
            topic::TopicSubscription::new(self.timeline_options, subscription),
        );
    }

    pub fn poll(&mut self, ts: timeline::Timestamp) {
        self.topics.iter_mut().for_each(|(_, topic)| topic.poll(ts))
    }
}
