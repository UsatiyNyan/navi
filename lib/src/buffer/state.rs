use std::collections::HashMap;

use super::record;
use super::timeline;
use super::topic;

pub struct State<'a, C: 'a> {
    topics: HashMap<String, Box<dyn topic::Topic<C> + 'a>>,
    timeline_options: timeline::TimelineOptions,
}

impl<'a, C> State<'a, C> {
    pub fn new(timeline_options: timeline::TimelineOptions) -> Self {
        Self {
            topics: Default::default(),
            timeline_options,
        }
    }

    pub fn subscribe<T, S>(&mut self, topic_path: String, subscription: S)
    where
        S: FnMut(&mut C, &timeline::Timeline<C, T>, timeline::Timestamp) + 'a,
        T: record::Record<C> + 'a,
    {
        self.topics.insert(
            topic_path,
            topic::TopicSubscription::new(self.timeline_options, subscription),
        );
    }

    pub fn poll(&mut self, completion: &mut C, ts: timeline::Timestamp) {
        self.topics
            .iter_mut()
            .for_each(|(_, topic)| topic.poll(completion, ts))
    }
}
