use super::record;
use super::timeline;

pub trait Topic {
    fn poll(&mut self, ts: timeline::Timestamp);
}

pub struct TopicSubscription<
    T: record::Record,
    S: FnMut(timeline::Timestamp, &timeline::Timeline<T>),
> {
    timeline: timeline::Timeline<T>,
    subscription: S,
}

impl<'a, T, S> TopicSubscription<T, S>
where
    T: record::Record + 'a,
    S: FnMut(timeline::Timestamp, &timeline::Timeline<T>) + 'a,
{
    pub fn new(
        timeline_options: timeline::TimelineOptions,
        subscription: S,
    ) -> Box<dyn Topic + 'a> {
        Box::new(Self {
            timeline: timeline::Timeline::new(timeline_options),
            subscription,
        })
    }
}

impl<T, S> Topic for TopicSubscription<T, S>
where
    T: record::Record,
    S: FnMut(timeline::Timestamp, &timeline::Timeline<T>),
{
    fn poll(&mut self, ts: timeline::Timestamp) {
        (self.subscription)(ts, &self.timeline);
    }
}
