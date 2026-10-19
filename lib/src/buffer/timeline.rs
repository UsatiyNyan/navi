pub use jiff::{SignedDuration as Duration, Timestamp};
use std::{collections::BTreeMap, ops::RangeBounds};

#[derive(Debug)]
pub struct Timeline<T> {
    tree: BTreeMap<Timestamp, T>,
}

impl<T> Default for Timeline<T> {
    fn default() -> Self {
        Self {
            tree: Default::default(),
        }
    }
}

impl<T> Timeline<T> {
    pub fn query_last(&self, ts: Timestamp) -> Option<(&Timestamp, &T)> {
        self.tree.range(..=ts).last()
    }

    pub fn query_next(&self, ts: Timestamp) -> Option<(&Timestamp, &T)> {
        use std::ops::Bound;
        self.tree
            .range((Bound::Excluded(ts), Bound::Unbounded))
            .next()
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

    pub fn query_around(
        &self,
        ts: Timestamp,
    ) -> (Option<(&Timestamp, &T)>, Option<(&Timestamp, &T)>) {
        (self.query_last(ts), self.query_next(ts))
    }

    pub fn insert(&mut self, ts: Timestamp, value: T) -> Option<T> {
        self.tree.insert(ts, value)
    }

    pub fn evict_around(&mut self, ts: Timestamp, window: Duration, margin: usize) {
        let lower_bound = ts.checked_sub(window).unwrap_or(Timestamp::MIN);
        let upper_bound = ts
            .checked_add(window)
            .and_then(|x| x.checked_add(Duration::from_nanos(1)))
            .ok();

        let first_kept = match margin.checked_sub(1) {
            Some(n) => self.tree.range(..lower_bound).nth_back(n).map(|(k, _)| *k),
            None => Some(lower_bound),
        };
        let first_dropped = upper_bound
            .and_then(|upper_bound| self.tree.range(upper_bound..).nth(margin).map(|(k, _)| *k));

        if let Some(first_kept) = first_kept {
            self.tree = self.tree.split_off(&first_kept);
        }
        if let Some(first_dropped) = first_dropped {
            let _ = self.tree.split_off(&first_dropped);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: i64) -> Timestamp {
        Timestamp::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn timeline(secs: &[i64]) -> Timeline<i64> {
        let mut timeline = Timeline::default();
        for &s in secs {
            timeline.insert(at(s), s);
        }
        timeline
    }

    fn keys(timeline: &Timeline<i64>) -> Vec<i64> {
        timeline
            .query_range_from_end(..)
            .rev()
            .map(|(_, v)| *v)
            .collect()
    }

    #[test]
    fn out_of_order_inserts_are_sorted() {
        let timeline = timeline(&[3, 1, 4, 2, 0]);
        assert_eq!(keys(&timeline), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn insert_returns_previous_value_on_same_ts() {
        let mut timeline = Timeline::default();
        assert_eq!(timeline.insert(at(1), "a"), None);
        assert_eq!(timeline.insert(at(1), "b"), Some("a"));
        assert_eq!(timeline.query_last(at(1)), Some((&at(1), &"b")));
    }

    #[test]
    fn query_last_on_empty() {
        let timeline = Timeline::<i64>::default();
        assert_eq!(timeline.query_last(at(0)), None);
    }

    #[test]
    fn query_last() {
        let timeline = timeline(&[10, 20, 30]);
        assert_eq!(timeline.query_last(at(9)), None);
        assert_eq!(timeline.query_last(at(10)), Some((&at(10), &10)));
        assert_eq!(timeline.query_last(at(25)), Some((&at(20), &20)));
        assert_eq!(timeline.query_last(at(30)), Some((&at(30), &30)));
        assert_eq!(timeline.query_last(at(99)), Some((&at(30), &30)));
    }

    #[test]
    fn query_next_on_empty() {
        let timeline = Timeline::<i64>::default();
        assert_eq!(timeline.query_next(at(0)), None);
    }

    #[test]
    fn query_next_is_strictly_after() {
        let timeline = timeline(&[10, 20, 30]);
        assert_eq!(timeline.query_next(at(0)), Some((&at(10), &10)));
        assert_eq!(timeline.query_next(at(10)), Some((&at(20), &20)));
        assert_eq!(timeline.query_next(at(25)), Some((&at(30), &30)));
        assert_eq!(timeline.query_next(at(30)), None);
        assert_eq!(timeline.query_next(at(99)), None);
    }

    #[test]
    fn query_next_skips_sample_at_exact_ts_by_one_nanosecond() {
        let ns = Duration::from_nanos(1);
        let mut timeline = Timeline::default();
        timeline.insert(at(1), "at");
        timeline.insert(at(1) + ns, "after");
        assert_eq!(timeline.query_next(at(1)), Some((&(at(1) + ns), &"after")));
        assert_eq!(timeline.query_next(at(1) - ns), Some((&at(1), &"at")));
    }

    #[test]
    fn query_next_at_timestamp_bounds() {
        let mut timeline = Timeline::default();
        timeline.insert(Timestamp::MIN, "min");
        timeline.insert(Timestamp::MAX, "max");
        assert_eq!(
            timeline.query_next(Timestamp::MIN),
            Some((&Timestamp::MAX, &"max"))
        );
        // nothing is strictly after MAX, even a sample at MAX itself
        assert_eq!(timeline.query_next(Timestamp::MAX), None);
    }

    #[test]
    fn query_around_on_empty() {
        let timeline = Timeline::<i64>::default();
        assert_eq!(timeline.query_around(at(0)), (None, None));
    }

    #[test]
    fn query_around_brackets_ts() {
        let timeline = timeline(&[10, 20, 30]);
        // before the first sample
        assert_eq!(timeline.query_around(at(5)), (None, Some((&at(10), &10))));
        // between samples
        assert_eq!(
            timeline.query_around(at(15)),
            (Some((&at(10), &10)), Some((&at(20), &20)))
        );
        // exact hit: last is the hit, next is the following sample
        assert_eq!(
            timeline.query_around(at(20)),
            (Some((&at(20), &20)), Some((&at(30), &30)))
        );
        // exact hit on the newest sample
        assert_eq!(timeline.query_around(at(30)), (Some((&at(30), &30)), None));
        // after the last sample
        assert_eq!(timeline.query_around(at(99)), (Some((&at(30), &30)), None));
    }

    #[test]
    fn query_around_single_sample() {
        let timeline = timeline(&[10]);
        assert_eq!(timeline.query_around(at(9)), (None, Some((&at(10), &10))));
        assert_eq!(timeline.query_around(at(10)), (Some((&at(10), &10)), None));
        assert_eq!(timeline.query_around(at(11)), (Some((&at(10), &10)), None));
    }

    #[test]
    fn query_around_never_returns_the_same_sample_twice() {
        let timeline = timeline(&[1, 2, 3]);
        for s in 0..=4 {
            if let (Some((last, _)), Some((next, _))) = timeline.query_around(at(s)) {
                assert!(last < next, "at {s}: {last} !< {next}");
                assert!(*last <= at(s) && at(s) < *next);
            }
        }
    }

    #[test]
    fn query_range_from_end_is_newest_first_and_respects_bounds() {
        let timeline = timeline(&[1, 2, 3, 4, 5]);
        let collect = |it: &mut dyn Iterator<Item = (&Timestamp, &i64)>| {
            it.map(|(_, v)| *v).collect::<Vec<_>>()
        };
        assert_eq!(
            collect(&mut timeline.query_range_from_end(..)),
            [5, 4, 3, 2, 1]
        );
        assert_eq!(
            collect(&mut timeline.query_range_from_end(at(2)..at(4))),
            [3, 2]
        );
        assert_eq!(
            collect(&mut timeline.query_range_from_end(at(2)..=at(4))),
            [4, 3, 2]
        );
        assert_eq!(collect(&mut timeline.query_range_from_end(..at(3))), [2, 1]);
        assert_eq!(collect(&mut timeline.query_range_from_end(at(6)..)), []);
    }

    #[test]
    fn evict_around_keeps_inclusive_window() {
        let mut timeline = timeline(&[0, 1, 2, 3, 4, 5, 6]);
        timeline.evict_around(at(3), Duration::from_secs(2), 0);
        assert_eq!(keys(&timeline), [1, 2, 3, 4, 5]);
    }

    #[test]
    fn evict_around_drops_entries_one_nanosecond_outside() {
        let window = Duration::from_secs(1);
        let mut timeline = Timeline::default();
        let lower = at(10) - window;
        let upper = at(10) + window;
        let ns = Duration::from_nanos(1);
        for ts in [lower - ns, lower, upper, upper + ns] {
            timeline.insert(ts, ());
        }
        timeline.evict_around(at(10), window, 0);
        let kept: Vec<_> = timeline
            .query_range_from_end(..)
            .rev()
            .map(|(ts, _)| *ts)
            .collect();
        assert_eq!(kept, [lower, upper]);
    }

    #[test]
    fn evict_around_zero_window_keeps_only_exact_ts() {
        let mut timeline = timeline(&[1, 2, 3]);
        timeline.evict_around(at(2), Duration::ZERO, 0);
        assert_eq!(keys(&timeline), [2]);
    }

    #[test]
    fn evict_around_far_from_data_empties() {
        let mut before = timeline(&[10, 11, 12]);
        before.evict_around(at(100), Duration::from_secs(5), 0);
        assert_eq!(keys(&before), []);

        let mut after = timeline(&[10, 11, 12]);
        after.evict_around(at(0), Duration::from_secs(5), 0);
        assert_eq!(keys(&after), []);
    }

    #[test]
    fn evict_around_is_idempotent_and_handles_empty() {
        let mut empty = Timeline::<i64>::default();
        empty.evict_around(at(0), Duration::from_secs(1), 0);
        assert_eq!(keys(&empty), []);

        let mut timeline = timeline(&[0, 1, 2, 3, 4]);
        timeline.evict_around(at(2), Duration::from_secs(1), 0);
        timeline.evict_around(at(2), Duration::from_secs(1), 0);
        assert_eq!(keys(&timeline), [1, 2, 3]);
    }

    #[test]
    fn evict_around_overflowing_window_keeps_everything() {
        let mut timeline = timeline(&[0, 1, 2]);
        timeline.evict_around(at(0), Duration::MAX, 0);
        assert_eq!(keys(&timeline), [0, 1, 2]);
    }

    #[test]
    fn evict_around_clamps_only_the_overflowing_side() {
        let window = Duration::from_secs(10);
        let near_min = Timestamp::MIN + Duration::from_secs(1);
        let near_max = Timestamp::MAX - Duration::from_secs(1);

        let mut low = timeline(&[0, 1, 2]);
        low.insert(near_min, -1);
        low.evict_around(near_min, window, 0);
        assert_eq!(keys(&low), [-1]);

        let mut high = timeline(&[0, 1, 2]);
        high.insert(near_max, -1);
        high.evict_around(near_max, window, 0);
        assert_eq!(keys(&high), [-1]);
    }

    #[test]
    fn evict_around_keeps_sample_at_timestamp_max() {
        let mut timeline = Timeline::default();
        timeline.insert(Timestamp::MIN, "min");
        timeline.insert(Timestamp::MAX, "max");
        timeline.evict_around(Timestamp::MAX, Duration::ZERO, 0);
        let kept: Vec<_> = timeline.query_range_from_end(..).map(|(_, v)| *v).collect();
        assert_eq!(kept, ["max"]);
    }

    #[test]
    fn insert_after_evict_outside_window_is_kept_until_next_evict() {
        let mut timeline = timeline(&[1, 2, 3]);
        timeline.evict_around(at(2), Duration::from_secs(1), 0);
        timeline.insert(at(50), 50);
        assert_eq!(keys(&timeline), [1, 2, 3, 50]);
        timeline.evict_around(at(2), Duration::from_secs(1), 0);
        assert_eq!(keys(&timeline), [1, 2, 3]);
    }

    fn evicted(secs: &[i64], ts: i64, window: i64, margin: usize) -> Vec<i64> {
        let mut timeline = timeline(secs);
        timeline.evict_around(at(ts), Duration::from_secs(window), margin);
        keys(&timeline)
    }

    #[test]
    fn evict_around_margin_keeps_that_many_samples_on_each_side() {
        let all = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(evicted(&all, 4, 1, 0), [3, 4, 5]);
        assert_eq!(evicted(&all, 4, 1, 1), [2, 3, 4, 5, 6]);
        assert_eq!(evicted(&all, 4, 1, 2), [1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn evict_around_margin_larger_than_available_keeps_everything_on_that_side() {
        let all = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(evicted(&all, 4, 1, 99), all);
        // only the lower side is short
        assert_eq!(evicted(&[2, 3, 4, 5, 6, 7, 8], 4, 1, 2), [2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn evict_around_margin_keeps_hold_sample_when_window_has_no_sample_before_ts() {
        // window [7, 17] holds 15 but nothing <= 12; hold-last at 12 needs 0
        let mut held = timeline(&[0, 15]);
        held.evict_around(at(12), Duration::from_secs(5), 1);
        assert_eq!(keys(&held), [0, 15]);
        assert_eq!(held.query_last(at(12)), Some((&at(0), &0)));

        let mut dropped = timeline(&[0, 15]);
        dropped.evict_around(at(12), Duration::from_secs(5), 0);
        assert_eq!(dropped.query_last(at(12)), None);
    }

    #[test]
    fn evict_around_margin_with_empty_window() {
        // cursor after the data: keep the newest for hold-last
        assert_eq!(evicted(&[10, 11, 12], 100, 5, 1), [12]);
        // cursor before the data: keep the oldest
        assert_eq!(evicted(&[10, 11, 12], 0, 5, 1), [10]);
        // cursor in a gap: one on each side, for interpolation
        assert_eq!(evicted(&[0, 1, 2, 20, 21, 22], 10, 2, 1), [2, 20]);
    }

    #[test]
    fn evict_around_margin_counts_strictly_outside_the_window() {
        // 3 sits exactly on the lower bound and 5 on the upper bound: both are inside,
        // so the margin adds 2 and 6 on top
        assert_eq!(evicted(&[1, 2, 3, 4, 5, 6, 7], 4, 1, 1), [2, 3, 4, 5, 6]);
    }

    #[test]
    fn evict_around_margin_on_empty_and_idempotent() {
        assert_eq!(evicted(&[], 0, 1, 3), Vec::<i64>::new());

        let mut twice = timeline(&[0, 1, 2, 3, 4, 5, 6, 7, 8]);
        twice.evict_around(at(4), Duration::from_secs(1), 1);
        twice.evict_around(at(4), Duration::from_secs(1), 1);
        assert_eq!(keys(&twice), [2, 3, 4, 5, 6]);
    }

    #[test]
    fn evict_around_margin_with_overflowing_window() {
        assert_eq!(evicted(&[0, 1, 2], 0, i64::MAX / 2, 1), [0, 1, 2]);

        // upper side overflows (nothing to cut), lower side still keeps one extra
        let near_max = Timestamp::MAX - Duration::from_secs(1);
        let mut high = timeline(&[0, 1, 2]);
        high.insert(near_max, -1);
        high.evict_around(near_max, Duration::from_secs(10), 1);
        assert_eq!(keys(&high), [2, -1]);
    }
}
