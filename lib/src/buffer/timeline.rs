use super::record::{Record, RecordNode};
use jiff::{SignedDuration as Duration, Timestamp};
use std::collections::{BTreeMap, btree_map::Entry};

pub struct Timeline<T: Record<T> + Clone> {
    tree: BTreeMap<Timestamp, RecordNode<T>>,
    window: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineOptions {
    window: Duration,
}

impl<T: Record<T> + Clone> Timeline<T> {
    pub const fn new(options: TimelineOptions) -> Self {
        Self {
            tree: BTreeMap::new(),
            window: options.window,
        }
    }

    pub fn query(&self, ts: Timestamp) -> Option<T> {
        let mut acc: Option<RecordNode<T>> = None;
        for (_, v) in self.tree.range(..=ts).rev() {
            acc = Some(match acc {
                Some(record) => record.combine(v),
                None => v.clone(),
            });
        }
        acc.map(|x| x.record_value)
    }

    pub fn insert(&mut self, ts: Timestamp, new_record_node: RecordNode<T>) {
        match self.tree.entry(ts) {
            Entry::Occupied(entry) => entry.into_mut().merge(new_record_node),
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
