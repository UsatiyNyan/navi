pub use jiff::{SignedDuration as Duration, Timestamp};
use std::{collections::BTreeMap, ops::RangeBounds};

pub struct Timeline<T> {
    tree: BTreeMap<Timestamp, T>,
}

impl<T> Timeline<T> {
    pub const fn new() -> Self {
        Self {
            tree: BTreeMap::new(),
        }
    }

    pub fn query_last(&self, ts: Timestamp) -> Option<(&Timestamp, &T)> {
        self.tree.range(..=ts).last()
    }

    pub fn query_range_from_end<'a, R>(
        &'a self,
        r: R,
    ) -> impl DoubleEndedIterator<Item = (&'a Timestamp, &'a T)>
    where
        R: RangeBounds<Timestamp>,
    {
        self.tree.range(r).rev()
    }

    pub fn insert(&mut self, ts: Timestamp, value: T) -> Option<T> {
        self.tree.insert(ts, value)
    }

    pub fn evict_around(&mut self, ts: Timestamp, window: Duration) {
        let lower_bound = ts - window;
        let upper_bound = ts + window + Duration::from_nanos(1);
        self.tree = self.tree.split_off(&lower_bound);
        let _ = self.tree.split_off(&upper_bound);
    }
}
