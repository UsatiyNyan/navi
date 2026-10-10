use super::record;
use super::timeline;
use std::marker::PhantomData;

pub trait Topic<C> {
    fn poll(&mut self, completion: &mut C, ts: timeline::Timestamp);
}

pub struct TopicSubscription<C, R, S>
where
    R: record::Record<C>,
    S: FnMut(&mut C, &timeline::Timeline<C, R>, timeline::Timestamp),
{
    timeline: timeline::Timeline<C, R>,
    subscription: S,
    _completion: PhantomData<C>,
}

impl<'a, C, R, S> TopicSubscription<C, R, S>
where
    C: 'a,
    R: record::Record<C> + 'a,
    S: FnMut(&mut C, &timeline::Timeline<C, R>, timeline::Timestamp) + 'a,
{
    pub fn new(
        timeline_options: timeline::TimelineOptions,
        subscription: S,
    ) -> Box<dyn Topic<C> + 'a> {
        Box::new(Self {
            timeline: timeline::Timeline::new(timeline_options),
            subscription,
            _completion: PhantomData {},
        })
    }
}

impl<C, R, S> Topic<C> for TopicSubscription<C, R, S>
where
    R: record::Record<C>,
    S: FnMut(&mut C, &timeline::Timeline<C, R>, timeline::Timestamp),
{
    fn poll(&mut self, completion: &mut C, ts: timeline::Timestamp) {
        (self.subscription)(completion, &self.timeline, ts)
    }
}
