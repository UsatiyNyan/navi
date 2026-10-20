use super::{Duration, Interner, Timestamp};

// repeats Timeline<T>
// TODO: procmacro to reduce boilerplate
pub trait TopicInterface {
    fn evict_around(&mut self, ts: Timestamp, window: Duration, margin: usize);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TopicId(u32);

impl From<TopicId> for u32 {
    fn from(value: TopicId) -> Self {
        value.0
    }
}

impl From<u32> for TopicId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

pub struct TopicStore<D> {
    names: Interner<TopicId>,
    items: Vec<D>,
}

impl<D> Default for TopicStore<D> {
    fn default() -> Self {
        Self {
            names: Default::default(),
            items: Default::default(),
        }
    }
}

impl<D> TopicStore<D> {
    pub fn entry<'a>(&'a mut self, topic: &'a str) -> TopicEntry<'a, D> {
        match self.names.lookup(topic) {
            Some(id) => TopicEntry::Occupied(id, self.get_mut(id)),
            None => TopicEntry::Vacant(VacantTopicEntry(self, topic)),
        }
    }
    pub fn get(&self, id: TopicId) -> &D {
        &self.items[Into::<u32>::into(id) as usize]
    }
    pub fn get_mut(&mut self, id: TopicId) -> &mut D {
        &mut self.items[Into::<u32>::into(id) as usize]
    }
    pub fn iter(&self) -> impl Iterator<Item = (TopicId, &D)> {
        std::iter::zip((0..self.len()).map(|x| TopicId(x as u32)), &self.items)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TopicId, &mut D)> {
        std::iter::zip((0..self.len()).map(|x| TopicId(x as u32)), &mut self.items)
    }
    pub fn lookup(&self, topic: &str) -> Option<TopicId> {
        self.names.lookup(topic)
    }
    pub fn name(&self, id: TopicId) -> &str {
        self.names.name(id)
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub enum TopicEntry<'a, D> {
    Occupied(TopicId, &'a mut D),
    Vacant(VacantTopicEntry<'a, D>),
}

pub struct VacantTopicEntry<'a, D>(&'a mut TopicStore<D>, &'a str);
impl<'a, D> VacantTopicEntry<'a, D> {
    pub fn insert(self, data: D) -> (TopicId, &'a mut D) {
        let (store, topic) = (self.0, self.1);
        debug_assert_eq!(store.names.len(), store.items.len());
        let id = store.names.intern(topic);
        debug_assert_eq!(Into::<u32>::into(id) as usize, store.items.len());
        (id, store.items.push_mut(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Timeline;

    #[derive(Debug)]
    enum TestData {
        A(Timeline<i32>),
        B(Timeline<&'static str>),
    }

    impl TopicInterface for TestData {
        fn evict_around(&mut self, ts: Timestamp, window: Duration, margin: usize) {
            match self {
                Self::A(tl) => tl.evict_around(ts, window, margin),
                Self::B(tl) => tl.evict_around(ts, window, margin),
            }
        }
    }

    fn at(secs: i64) -> Timestamp {
        Timestamp::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn timeline<T>(samples: &[(i64, T)]) -> Timeline<T>
    where
        T: Clone,
    {
        let mut tl = Timeline::default();
        for (s, v) in samples {
            tl.insert(at(*s), v.clone());
        }
        tl
    }

    fn a(samples: &[(i64, i32)]) -> TestData {
        TestData::A(timeline(samples))
    }

    fn b(samples: &[(i64, &'static str)]) -> TestData {
        TestData::B(timeline(samples))
    }

    fn values_a(d: &TestData) -> Vec<i32> {
        match d {
            TestData::A(tl) => tl.query_range_from_end(..).rev().map(|(_, v)| *v).collect(),
            other => panic!("expected A, got {other:?}"),
        }
    }

    fn values_b(d: &TestData) -> Vec<&'static str> {
        match d {
            TestData::B(tl) => tl.query_range_from_end(..).rev().map(|(_, v)| *v).collect(),
            other => panic!("expected B, got {other:?}"),
        }
    }

    fn store() -> TopicStore<TestData> {
        TopicStore {
            names: Interner::default(),
            items: Vec::new(),
        }
    }

    fn insert(store: &mut TopicStore<TestData>, topic: &str, d: TestData) -> TopicId {
        match store.entry(topic) {
            TopicEntry::Vacant(e) => e.insert(d).0,
            TopicEntry::Occupied(id, _) => panic!("{topic} already present as {id:?}"),
        }
    }

    #[test]
    fn empty() {
        let mut store = store();
        assert_eq!(store.len(), 0);
        assert_eq!(store.lookup("/a"), None);
        assert_eq!(store.iter().count(), 0);
        assert_eq!(store.iter_mut().count(), 0);
    }

    #[test]
    fn unknown_topic_is_vacant_and_nothing_changes_until_insert() {
        let mut store = store();
        assert!(matches!(store.entry("/a"), TopicEntry::Vacant(_)));
        assert_eq!(store.len(), 0);
        assert_eq!(store.lookup("/a"), None);
    }

    #[test]
    fn dropped_vacant_does_not_misalign_ids() {
        let mut store = store();
        let _ = store.entry("/rejected");
        let b = insert(&mut store, "/b", a(&[(0, 2)]));
        let rejected = insert(&mut store, "/rejected", a(&[(0, 1)]));

        assert_eq!(b, TopicId(0));
        assert_eq!(rejected, TopicId(1));
        assert_eq!(values_a(store.get(b)), [2]);
        assert_eq!(values_a(store.get(rejected)), [1]);
    }

    #[test]
    fn insert_assigns_sequential_ids_and_names_round_trip() {
        let mut store = store();
        let names = ["/lidar", "/mesh", "/markers"];
        let ids = names.map(|n| insert(&mut store, n, a(&[])));
        assert_eq!(ids, [TopicId(0), TopicId(1), TopicId(2)]);
        assert_eq!(store.len(), 3);
        for (id, name) in ids.into_iter().zip(names) {
            assert_eq!(store.name(id), name);
            assert_eq!(store.lookup(name), Some(id));
        }
    }

    #[test]
    fn insert_returns_reference_to_stored_data() {
        let mut store = store();
        let (id, d) = match store.entry("/a") {
            TopicEntry::Vacant(e) => e.insert(a(&[(0, 1)])),
            TopicEntry::Occupied(..) => unreachable!(),
        };
        let TestData::A(tl) = d else { unreachable!() };
        tl.insert(at(1), 2);
        assert_eq!(values_a(store.get(id)), [1, 2]);
    }

    #[test]
    fn known_topic_is_occupied_with_same_id_and_data() {
        let mut store = store();
        let id = insert(&mut store, "/a", b(&[(0, "x")]));
        match store.entry("/a") {
            TopicEntry::Occupied(occupied_id, d) => {
                assert_eq!(occupied_id, id);
                assert_eq!(values_b(d), ["x"]);
            }
            TopicEntry::Vacant(_) => panic!("expected Occupied"),
        }
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn mutation_through_occupied_and_get_mut_is_visible() {
        let mut store = store();
        let id = insert(&mut store, "/a", a(&[(0, 1)]));

        if let TopicEntry::Occupied(_, TestData::A(tl)) = store.entry("/a") {
            tl.insert(at(1), 2);
        } else {
            panic!("expected Occupied A");
        }
        if let TestData::A(tl) = store.get_mut(id) {
            tl.insert(at(2), 3);
        }
        assert_eq!(values_a(store.get(id)), [1, 2, 3]);
    }

    #[test]
    fn replacing_data_through_occupied_keeps_id() {
        let mut store = store();
        let id = insert(&mut store, "/a", a(&[(0, 1)]));
        let other = insert(&mut store, "/other", a(&[(0, 9)]));

        let TopicEntry::Occupied(_, d) = store.entry("/a") else {
            panic!("expected Occupied");
        };
        *d = b(&[(0, "replaced")]);

        assert_eq!(store.lookup("/a"), Some(id));
        assert_eq!(store.len(), 2);
        assert_eq!(values_b(store.get(id)), ["replaced"]);
        assert_eq!(values_a(store.get(other)), [9]);
    }

    #[test]
    fn iter_and_iter_mut_yield_arrival_order() {
        let mut store = store();
        insert(&mut store, "/c", a(&[(0, 3)]));
        insert(&mut store, "/a", a(&[(0, 1)]));
        insert(&mut store, "/b", a(&[(0, 2)]));

        let seen: Vec<_> = store
            .iter()
            .map(|(id, d)| (id, store.name(id), values_a(d)))
            .collect();
        assert_eq!(
            seen,
            [
                (TopicId(0), "/c", vec![3]),
                (TopicId(1), "/a", vec![1]),
                (TopicId(2), "/b", vec![2]),
            ]
        );

        for (id, d) in store.iter_mut() {
            if let TestData::A(tl) = d {
                tl.insert(at(1), 10 + Into::<u32>::into(id) as i32);
            }
        }
        assert_eq!(values_a(store.get(TopicId(0))), [3, 10]);
        assert_eq!(values_a(store.get(TopicId(2))), [2, 12]);
    }

    #[test]
    fn eviction_via_iter_mut_with_margin_chosen_per_variant() {
        let mut store = store();
        let busy = insert(&mut store, "/busy", a(&[(0, 0), (50, 50), (100, 100)]));
        let latched = insert(&mut store, "/latched", b(&[(0, "map")]));

        // app-side policy: B is latched data and keeps its last sample before the window
        for (_, d) in store.iter_mut() {
            let margin = match d {
                TestData::A(_) => 0,
                TestData::B(_) => 1,
            };
            d.evict_around(at(100), Duration::from_secs(10), margin);
        }

        assert_eq!(values_a(store.get(busy)), [100]);
        assert_eq!(values_b(store.get(latched)), ["map"]);
        assert_eq!(store.len(), 2);
        assert_eq!(store.lookup("/latched"), Some(latched));
    }
}
