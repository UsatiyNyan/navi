use super::record::{Record, RecordNode};
pub use jiff::{SignedDuration as Duration, Timestamp};
use std::collections::{BTreeMap, btree_map::Entry};

pub struct Timeline<C, R: Record<C>> {
    tree: BTreeMap<Timestamp, RecordNode<C, R>>,
    window: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineOptions {
    pub window: Duration,
}

impl<C, R: Record<C>> Timeline<C, R> {
    pub const fn new(options: TimelineOptions) -> Self {
        Self {
            tree: BTreeMap::new(),
            window: options.window,
        }
    }

    pub fn query(&self, completion: &mut C, ts: Timestamp) {
        self.query_with(
            completion,
            ts,
            |_, _| true,
            |_, v| v.record_type.is_snapshot(),
        )
    }

    pub fn query_with<P1, P2>(
        &self,
        completion: &mut C,
        ts: Timestamp,
        precondition: P1,
        postcondition: P2,
    ) where
        P1: Fn(&C, &RecordNode<C, R>) -> bool,
        P2: Fn(&C, &RecordNode<C, R>) -> bool,
    {
        for (_, v) in self.tree.range(..=ts).rev() {
            if !precondition(completion, v) {
                break;
            }
            v.record_value.accum(completion);
            if !postcondition(completion, v) {
                break;
            }
        }
    }

    pub fn insert(&mut self, ts: Timestamp, new_record_node: RecordNode<C, R>) {
        match self.tree.entry(ts) {
            Entry::Occupied(mut entry) => {
                let _ = entry.insert(new_record_node);
            }
            Entry::Vacant(entry) => {
                entry.insert(new_record_node);
                if let Some(first_entry) = self.tree.first_entry() {
                    if *first_entry.key() + self.window < ts {
                        first_entry.remove_entry();
                    } else if let Some(last_entry) = self.tree.last_entry() {
                        if ts + self.window < *last_entry.key() {
                            last_entry.remove_entry();
                        }
                    }
                }
            }
        }
    }
}
