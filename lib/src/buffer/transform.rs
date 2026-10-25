use super::{Duration, Interner, Timeline, Timestamp};
use smallvec::SmallVec;
use std::collections::{HashMap, hash_map};

type TransformMemo<T> = HashMap<(FrameId, Timestamp), Result<T, TransformError>>;
const INTERMEDIATE_DEPTH: usize = 8;
pub struct TransformBatchBase<'a, T, const MEMOIZE: bool> {
    store: &'a TransformStore<T>,
    root: FrameId,
    memo: TransformMemo<T>,
    requested: HashMap<(FrameId, Timestamp), u32>,
    accumulator: Vec<T>,

    intermediate_frames: SmallVec<[FrameId; INTERMEDIATE_DEPTH]>,
    intermediate_transforms: SmallVec<[T; INTERMEDIATE_DEPTH]>,
}
pub type TransformBatch<'a, T> = TransformBatchBase<'a, T, false>;

impl<'a, T: Transform, const MEMOIZE: bool> TransformBatchBase<'a, T, MEMOIZE> {
    pub fn new(store: &'a TransformStore<T>, root: FrameId) -> Self {
        Self {
            store,
            root,
            memo: Default::default(),
            requested: Default::default(),
            accumulator: Default::default(),
            intermediate_frames: Default::default(),
            intermediate_transforms: Default::default(),
        }
    }
    pub fn request(&mut self, child: FrameId, ts: Timestamp) -> Result<u32, TransformError> {
        let key = (child, ts);
        let requested_entry = match self.requested.entry(key) {
            hash_map::Entry::Occupied(entry) => return Ok(*entry.get()),
            hash_map::Entry::Vacant(entry) => entry,
        };

        if let hash_map::Entry::Occupied(entry) = self.memo.entry(key) {
            return Self::finalize(&mut self.accumulator, requested_entry, *entry.get());
        };

        Self::root_walk(self.store, &mut self.memo, self.root, ts);
        let resolve_result = Self::resolve(
            self.store,
            &mut self.memo,
            &mut self.intermediate_frames,
            &mut self.intermediate_transforms,
            child,
            ts,
        );
        Self::finalize(&mut self.accumulator, requested_entry, resolve_result)
    }

    pub fn response(&self) -> &[T] {
        &self.accumulator
    }

    fn root_walk(
        store: &TransformStore<T>,
        memo: &mut TransformMemo<T>,
        root: FrameId,
        ts: Timestamp,
    ) {
        match memo.entry((root, ts)) {
            hash_map::Entry::Occupied(_) => (), // expect Ok(T::identity())
            hash_map::Entry::Vacant(entry) => {
                entry.insert(Ok(T::identity()));

                let (mut ancestor, mut transform) = (root, T::identity());
                loop {
                    (ancestor, transform) = match store.edge_at(ancestor, ts) {
                        Ok(Some(edge)) => (edge.parent, T::compose(edge.transform, transform)),
                        Ok(None) | Err(_) => break,
                    };

                    match memo.entry((ancestor, ts)) {
                        hash_map::Entry::Occupied(_) => break,
                        hash_map::Entry::Vacant(entry) => entry.insert(Ok(transform.inverse())),
                    };
                }
            }
        }
    }

    fn resolve(
        store: &TransformStore<T>,
        memo: &mut TransformMemo<T>,
        intermediate_frames: &mut SmallVec<[FrameId; INTERMEDIATE_DEPTH]>,
        intermediate_transforms: &mut SmallVec<[T; INTERMEDIATE_DEPTH]>,
        child: FrameId,
        ts: Timestamp,
    ) -> Result<T, TransformError> {
        if let Some(entry) = memo.get(&(child, ts)) {
            return *entry;
        }

        intermediate_frames.clear();
        intermediate_transforms.clear();

        let mut ancestor = child;
        let last_known_transform_result = loop {
            if intermediate_frames.contains(&ancestor) {
                break Err(TransformError::Cycle);
            }

            let edge = match store.edge_at(ancestor, ts) {
                Ok(Some(edge)) => edge,
                Ok(None) => break Err(TransformError::NoPath),
                Err(TransformEdgeAtError::NoSampleBefore) => {
                    break Err(TransformError::NoSampleBefore { child: ancestor });
                }
            };
            intermediate_frames.push(ancestor);
            intermediate_transforms.push(edge.transform);
            ancestor = edge.parent;

            if let Some(entry) = memo.get(&(ancestor, ts)) {
                break *entry;
            }
        };

        let intermediate_range = std::iter::zip(intermediate_frames, intermediate_transforms).rev();
        let result = match last_known_transform_result {
            Ok(mut accum) => {
                for (f, tf) in intermediate_range {
                    accum = T::compose(accum, *tf);
                    if MEMOIZE {
                        memo.insert((*f, ts), Ok(accum));
                    }
                }
                Ok(accum)
            }
            Err(err) => {
                if MEMOIZE {
                    for (f, _) in intermediate_range {
                        memo.insert((*f, ts), Err(err));
                    }
                }
                Err(err)
            }
        };
        // with MEMOIZE requested child would be inserted as the 0-th item in intermediate_transforms
        // without MEMOIZE, the accumulated transform needs to be added manually
        if !MEMOIZE {
            memo.insert((child, ts), result);
        }
        result
    }

    fn finalize(
        accumulator: &mut Vec<T>,
        requested_entry: hash_map::VacantEntry<(FrameId, Timestamp), u32>,
        result: Result<T, TransformError>,
    ) -> Result<u32, TransformError> {
        match result {
            Ok(value) => {
                let index = accumulator.len() as u32;
                accumulator.push(value);
                requested_entry.insert(index);
                Ok(index)
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransformError {
    NoSampleBefore { child: FrameId },
    NoPath,
    Cycle,
}

pub struct TransformStore<T> {
    frames: Interner<FrameId>,
    edges: Vec<Option<TransformEdges<T>>>,
}

impl<T> TransformStore<T> {
    pub fn intern(&mut self, frame: &str) -> FrameId {
        match self.lookup(frame) {
            Some(id) => id,
            None => {
                debug_assert_eq!(self.frames.len(), self.edges.len());
                let id = self.frames.intern(frame);
                debug_assert_eq!(Into::<u32>::into(id) as usize, self.edges.len());
                self.edges.push(None);
                id
            }
        }
    }
    pub fn lookup(&self, frame: &str) -> Option<FrameId> {
        self.frames.lookup(frame)
    }
    pub fn name(&self, id: FrameId) -> &str {
        self.frames.name(id)
    }
    // transform is static if ts is None
    // returns previous edges if changed
    pub fn insert(
        &mut self,
        child: FrameId,
        parent: FrameId,
        ts: Option<Timestamp>,
        transform: T,
    ) -> Option<TransformEdges<T>> {
        let new_edge = TransformEdge { parent, transform };
        let slot = self.get_mut(child);
        if let (Some(TransformEdges::Dynamic(timeline)), Some(ts)) = (slot.as_mut(), ts) {
            timeline.insert(ts, new_edge);
            return None;
        }
        let prev = slot.replace(match ts {
            Some(ts) => TransformEdges::Dynamic(Timeline::from_iter([(ts, new_edge)])),
            None => TransformEdges::Static(new_edge),
        });
        match (&prev, ts) {
            (Some(TransformEdges::Static(_)), None) => None,
            _ => prev,
        }
    }
    pub fn evict_around(&mut self, ts: Timestamp, window: Duration, margin: usize) {
        debug_assert!(margin >= 1, "at least 1 transform is expected to survive");
        for edges in &mut self.edges {
            if let Some(TransformEdges::Dynamic(timeline)) = edges {
                timeline.evict_around(ts, window, margin)
            }
        }
    }
    // TODO: Option<&TE<T>>
    fn get(&self, child: FrameId) -> &Option<TransformEdges<T>> {
        &self.edges[Into::<u32>::into(child) as usize]
    }
    fn get_mut(&mut self, child: FrameId) -> &mut Option<TransformEdges<T>> {
        &mut self.edges[Into::<u32>::into(child) as usize]
    }
}

impl<T: Transform> TransformStore<T> {
    pub fn edge_at(
        &self,
        child: FrameId,
        ts: Timestamp,
    ) -> Result<Option<TransformEdge<T>>, TransformEdgeAtError> {
        match self.get(child) {
            None => Ok(None),
            Some(TransformEdges::Static(edge)) => Ok(Some(*edge)),
            Some(TransformEdges::Dynamic(timeline)) => match timeline.query_around(ts) {
                (Some((&before_ts, &before_edge)), _) if before_ts == ts => Ok(Some(before_edge)),
                (Some((_, &before_edge)), None) => Ok(Some(before_edge)),
                (Some((_, &before_edge)), Some((_, &after_edge)))
                    if before_edge.parent != after_edge.parent =>
                {
                    Ok(Some(before_edge))
                }
                (Some((&before_ts, &before_edge)), Some((&after_ts, &after_edge))) => {
                    debug_assert_eq!(before_edge.parent, after_edge.parent);

                    Ok(Some(TransformEdge {
                        parent: before_edge.parent,
                        transform: T::interpolate(
                            before_edge.transform,
                            after_edge.transform,
                            ts.duration_since(before_ts).as_nanos() as f64
                                / after_ts.duration_since(before_ts).as_nanos() as f64,
                        ),
                    }))
                }
                (None, _) => Err(TransformEdgeAtError::NoSampleBefore),
            },
        }
    }
}

#[derive(Debug)]
pub enum TransformEdgeAtError {
    NoSampleBefore,
}

pub enum TransformEdges<T> {
    Static(TransformEdge<T>),
    Dynamic(Timeline<TransformEdge<T>>),
}

#[derive(Copy, Clone, Debug)]
pub struct TransformEdge<T> {
    pub parent: FrameId,
    pub transform: T,
}

pub trait Transform: Copy {
    fn identity() -> Self;
    // parent <- child * child <- x = parent <- x
    fn compose(parent: Self, child: Self) -> Self;
    fn inverse(self) -> Self;
    // t in [0, 1]; t = 0 -> a, t = 1 -> b
    fn interpolate(a: Self, b: Self, t: f64) -> Self;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FrameId(u32);

impl From<FrameId> for u32 {
    fn from(value: FrameId) -> Self {
        value.0
    }
}

impl From<u32> for FrameId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1D affine stand-in `x -> scale * x + offset`. Unlike a pure offset it doesn't
    /// commute, so tests catch a swapped `compose(parent, child)`. Values are kept to
    /// powers of two so results compare exactly.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub(crate) struct Affine1 {
        pub scale: f64,
        pub offset: f64,
    }

    impl Affine1 {
        pub fn new(scale: f64, offset: f64) -> Self {
            Self { scale, offset }
        }
        pub fn apply(self, x: f64) -> f64 {
            self.scale * x + self.offset
        }
    }

    impl Transform for Affine1 {
        fn identity() -> Self {
            Self::new(1.0, 0.0)
        }
        fn compose(parent: Self, child: Self) -> Self {
            Self::new(
                parent.scale * child.scale,
                parent.scale * child.offset + parent.offset,
            )
        }
        fn inverse(self) -> Self {
            Self::new(1.0 / self.scale, -self.offset / self.scale)
        }
        fn interpolate(a: Self, b: Self, t: f64) -> Self {
            Self::new(
                a.scale + (b.scale - a.scale) * t,
                a.offset + (b.offset - a.offset) * t,
            )
        }
    }

    #[test]
    fn frame_id_round_trips_through_u32() {
        assert_eq!(u32::from(FrameId::from(7)), 7);
        assert_eq!(FrameId::from(0), FrameId::from(0));
        assert_ne!(FrameId::from(0), FrameId::from(1));
    }

    // The checks below pin down the conventions documented on `Transform`
    // using the stand-in, so later TfBuffer/FrameTable tests can rely on it.

    #[test]
    fn compose_applies_child_first() {
        let parent = Affine1::new(2.0, 1.0);
        let child = Affine1::new(4.0, 2.0);
        let x = 5.0;
        // parent <- child * child <- x = parent <- x
        assert_eq!(
            Affine1::compose(parent, child).apply(x),
            parent.apply(child.apply(x))
        );
        assert_ne!(
            Affine1::compose(parent, child),
            Affine1::compose(child, parent)
        );
    }

    #[test]
    fn identity_is_neutral() {
        let a = Affine1::new(2.0, 1.0);
        assert_eq!(Affine1::compose(Affine1::identity(), a), a);
        assert_eq!(Affine1::compose(a, Affine1::identity()), a);
    }

    #[test]
    fn inverse_cancels_on_both_sides() {
        let a = Affine1::new(4.0, -2.0);
        assert_eq!(Affine1::compose(a, a.inverse()), Affine1::identity());
        assert_eq!(Affine1::compose(a.inverse(), a), Affine1::identity());
    }

    #[test]
    fn root_relative_via_inverse_of_top() {
        // top <- root and top <- x known; root <- x = inverse(top <- root) * top <- x
        let top_root = Affine1::new(2.0, 8.0);
        let root_x = Affine1::new(4.0, 1.0);
        let top_x = Affine1::compose(top_root, root_x);
        assert_eq!(Affine1::compose(top_root.inverse(), top_x), root_x);
    }

    #[test]
    fn interpolate_endpoints_and_midpoint() {
        let a = Affine1::new(1.0, 0.0);
        let b = Affine1::new(3.0, 4.0);
        assert_eq!(Affine1::interpolate(a, b, 0.0), a);
        assert_eq!(Affine1::interpolate(a, b, 1.0), b);
        assert_eq!(Affine1::interpolate(a, b, 0.5), Affine1::new(2.0, 2.0));
    }

    // --- TransformStore ---
    //
    // `edge_at` answers one hop: the parent <- child edge at a time. Walking up to the
    // root (memoized, cycles, NoPath, inverse for a mid-tree root) is FrameTable's job.

    fn at(secs: i64) -> Timestamp {
        Timestamp::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn store() -> TransformStore<Affine1> {
        TransformStore {
            frames: Interner::default(),
            edges: Vec::new(),
        }
    }

    /// `edge_at` that must succeed, flattened to `(parent, transform)`.
    fn edge(
        store: &TransformStore<Affine1>,
        child: FrameId,
        ts: Timestamp,
    ) -> Option<(FrameId, Affine1)> {
        match store.edge_at(child, ts) {
            Ok(edge) => edge.map(|e| (e.parent, e.transform)),
            Err(_) => panic!("expected Ok at {ts}"),
        }
    }

    fn no_sample_before(store: &TransformStore<Affine1>, child: FrameId, ts: Timestamp) -> bool {
        matches!(
            store.edge_at(child, ts),
            Err(TransformEdgeAtError::NoSampleBefore)
        )
    }

    fn kind(previous: Option<TransformEdges<Affine1>>) -> &'static str {
        match previous {
            None => "none",
            Some(TransformEdges::Static(_)) => "static",
            Some(TransformEdges::Dynamic(_)) => "dynamic",
        }
    }

    #[test]
    fn intern_assigns_ids_and_names_round_trip() {
        let mut store = store();
        let map = store.intern("map");
        let odom = store.intern("odom");
        assert_eq!(store.intern("map"), map);
        assert_ne!(map, odom);
        assert_eq!(store.lookup("odom"), Some(odom));
        assert_eq!(store.lookup("base_link"), None);
        assert_eq!(store.name(map), "map");
    }

    #[test]
    fn interned_frame_that_was_never_a_child_has_no_parent() {
        let mut store = store();
        let map = store.intern("map");
        assert_eq!(edge(&store, map, at(0)), None);
        assert_eq!(edge(&store, map, Timestamp::MIN), None);
    }

    #[test]
    fn parent_of_an_edge_is_a_tree_top() {
        let mut store = store();
        let (map, odom) = (store.intern("map"), store.intern("odom"));
        store.insert(odom, map, None, Affine1::new(2.0, 1.0));
        assert_eq!(edge(&store, map, at(0)), None);
    }

    #[test]
    fn static_edge_resolves_at_any_time() {
        let mut store = store();
        let (base, laser) = (store.intern("base_link"), store.intern("laser"));
        let t = Affine1::new(2.0, 1.0);
        store.insert(laser, base, None, t);
        for ts in [Timestamp::MIN, at(-5), at(0), at(1_000), Timestamp::MAX] {
            assert_eq!(edge(&store, laser, ts), Some((base, t)));
        }
    }

    #[test]
    fn edge_at_returns_only_one_hop() {
        // map <- odom <- base_link: base_link's edge is parent <- child only, not map <- base_link
        let mut store = store();
        let (map, odom, base) = (
            store.intern("map"),
            store.intern("odom"),
            store.intern("base_link"),
        );
        let map_odom = Affine1::new(2.0, 1.0);
        let odom_base = Affine1::new(4.0, 2.0);
        store.insert(odom, map, None, map_odom);
        store.insert(base, odom, None, odom_base);
        assert_eq!(edge(&store, base, at(0)), Some((odom, odom_base)));
        assert_eq!(edge(&store, odom, at(0)), Some((map, map_odom)));
    }

    #[test]
    fn dynamic_exact_hit_returns_that_sample() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        let (a, b) = (Affine1::new(1.0, 0.0), Affine1::new(2.0, 8.0));
        store.insert(base, odom, Some(at(10)), a);
        store.insert(base, odom, Some(at(20)), b);
        assert_eq!(edge(&store, base, at(10)), Some((odom, a)));
        assert_eq!(edge(&store, base, at(20)), Some((odom, b)));
    }

    #[test]
    fn dynamic_between_samples_interpolates() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        store.insert(base, odom, Some(at(0)), Affine1::new(1.0, 0.0));
        store.insert(base, odom, Some(at(4)), Affine1::new(5.0, 8.0));
        // t = (1 - 0) / (4 - 0) = 0.25
        assert_eq!(
            edge(&store, base, at(1)),
            Some((odom, Affine1::new(2.0, 2.0)))
        );
        // t = 0.5
        assert_eq!(
            edge(&store, base, at(2)),
            Some((odom, Affine1::new(3.0, 4.0)))
        );
    }

    #[test]
    fn dynamic_interpolation_uses_the_bracketing_pair() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        store.insert(base, odom, Some(at(0)), Affine1::new(1.0, 0.0));
        store.insert(base, odom, Some(at(10)), Affine1::new(1.0, 100.0));
        store.insert(base, odom, Some(at(14)), Affine1::new(1.0, 108.0));
        // between 10 and 14, t = 0.5; not between 0 and 14
        assert_eq!(
            edge(&store, base, at(12)),
            Some((odom, Affine1::new(1.0, 104.0)))
        );
    }

    #[test]
    fn dynamic_after_last_sample_holds_it() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        let last = Affine1::new(2.0, 4.0);
        store.insert(base, odom, Some(at(0)), Affine1::new(1.0, 0.0));
        store.insert(base, odom, Some(at(10)), last);
        assert_eq!(edge(&store, base, at(11)), Some((odom, last)));
        assert_eq!(edge(&store, base, Timestamp::MAX), Some((odom, last)));
    }

    #[test]
    fn dynamic_before_first_sample_is_no_sample_before() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        store.insert(base, odom, Some(at(10)), Affine1::new(1.0, 0.0));
        assert!(no_sample_before(&store, base, at(9)));
        assert!(no_sample_before(&store, base, Timestamp::MIN));
        let one_ns_early = at(10) - Duration::from_nanos(1);
        assert!(no_sample_before(&store, base, one_ns_early));
    }

    #[test]
    fn reparent_between_samples_does_not_interpolate() {
        let mut store = store();
        let (map, odom, base) = (
            store.intern("map"),
            store.intern("odom"),
            store.intern("base_link"),
        );
        let under_odom = Affine1::new(1.0, 0.0);
        let under_map = Affine1::new(4.0, 40.0);
        store.insert(base, odom, Some(at(0)), under_odom);
        store.insert(base, map, Some(at(10)), under_map);
        // between: the earlier sample, parent and transform unchanged
        assert_eq!(edge(&store, base, at(5)), Some((odom, under_odom)));
        // on and after the reparent: the new parent
        assert_eq!(edge(&store, base, at(10)), Some((map, under_map)));
        assert_eq!(edge(&store, base, at(20)), Some((map, under_map)));
    }

    #[test]
    fn edges_of_different_children_are_independent() {
        let mut store = store();
        let (odom, base, laser) = (
            store.intern("odom"),
            store.intern("base_link"),
            store.intern("laser"),
        );
        store.insert(base, odom, Some(at(10)), Affine1::new(1.0, 0.0));
        store.insert(laser, base, None, Affine1::new(2.0, 0.0));
        assert!(no_sample_before(&store, base, at(0)));
        assert_eq!(
            edge(&store, laser, at(0)),
            Some((base, Affine1::new(2.0, 0.0)))
        );
    }

    #[test]
    fn insert_reports_the_previous_kind_only_when_it_changes() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        let t = Affine1::identity();
        assert_eq!(kind(store.insert(base, odom, Some(at(0)), t)), "none");
        assert_eq!(kind(store.insert(base, odom, Some(at(1)), t)), "none");
        assert_eq!(kind(store.insert(base, odom, None, t)), "dynamic");
        assert_eq!(kind(store.insert(base, odom, Some(at(2)), t)), "static");
    }

    #[test]
    fn static_over_static_overwrites_and_reports_nothing() {
        let mut store = store();
        let (base, laser) = (store.intern("base_link"), store.intern("laser"));
        let (old, new) = (Affine1::new(2.0, 1.0), Affine1::new(4.0, 2.0));
        assert_eq!(kind(store.insert(laser, base, None, old)), "none");
        // same kind: not a diagnostic, like dynamic after dynamic
        assert_eq!(kind(store.insert(laser, base, None, new)), "none");
        assert_eq!(edge(&store, laser, at(0)), Some((base, new)));
    }

    #[test]
    fn static_replaces_dynamic_history_and_vice_versa() {
        let mut store = store();
        let (odom, base) = (store.intern("odom"), store.intern("base_link"));
        let s = Affine1::new(2.0, 2.0);
        store.insert(base, odom, Some(at(0)), Affine1::new(1.0, 0.0));
        store.insert(base, odom, None, s);
        // static now answers everywhere, including where the old history had nothing
        assert_eq!(edge(&store, base, at(-100)), Some((odom, s)));

        let d = Affine1::new(4.0, 4.0);
        store.insert(base, odom, Some(at(50)), d);
        // the static value is gone: before the only dynamic sample there is nothing
        assert!(no_sample_before(&store, base, at(49)));
        assert_eq!(edge(&store, base, at(50)), Some((odom, d)));
    }

    #[test]
    fn eviction_keeps_hold_sample_of_a_quiet_edge_and_static_edges() {
        let mut store = store();
        let (map, odom, base) = (
            store.intern("map"),
            store.intern("odom"),
            store.intern("base_link"),
        );
        let last = Affine1::new(2.0, 1.0);
        let fixed = Affine1::new(4.0, 0.0);
        store.insert(base, odom, Some(at(0)), Affine1::new(1.0, 0.0));
        store.insert(base, odom, Some(at(1)), last);
        store.insert(odom, map, None, fixed);

        store.evict_around(at(100), Duration::from_secs(5), 1);

        assert_eq!(edge(&store, base, at(100)), Some((odom, last)));
        assert_eq!(edge(&store, odom, at(100)), Some((map, fixed)));
    }

    // --- TransformBatch ---
    //
    //   map <- odom <- base_link <- laser
    //                  base_link <- camera
    //          odom <- imu
    //
    // All values are dyadic, so any composition order gives bit-identical results.

    const MAP_ODOM: Affine1 = Affine1 {
        scale: 2.0,
        offset: 1.0,
    };
    const ODOM_BASE: Affine1 = Affine1 {
        scale: 4.0,
        offset: 2.0,
    };
    const BASE_LASER: Affine1 = Affine1 {
        scale: 2.0,
        offset: 3.0,
    };
    const BASE_CAMERA: Affine1 = Affine1 {
        scale: 0.5,
        offset: -1.0,
    };
    const ODOM_IMU: Affine1 = Affine1 {
        scale: 1.0,
        offset: 5.0,
    };

    struct Tree {
        store: TransformStore<Affine1>,
        map: FrameId,
        odom: FrameId,
        base: FrameId,
        laser: FrameId,
        camera: FrameId,
        imu: FrameId,
    }

    /// The tree above. `map <- odom` and `odom <- base_link` are static unless a test
    /// overrides them.
    fn tree() -> Tree {
        let mut store = store();
        let map = store.intern("map");
        let odom = store.intern("odom");
        let base = store.intern("base_link");
        let laser = store.intern("laser");
        let camera = store.intern("camera");
        let imu = store.intern("imu");
        store.insert(odom, map, None, MAP_ODOM);
        store.insert(base, odom, None, ODOM_BASE);
        store.insert(laser, base, None, BASE_LASER);
        store.insert(camera, base, None, BASE_CAMERA);
        store.insert(imu, odom, None, ODOM_IMU);
        Tree {
            store,
            map,
            odom,
            base,
            laser,
            camera,
            imu,
        }
    }

    fn compose(transforms: &[Affine1]) -> Affine1 {
        transforms
            .iter()
            .fold(Affine1::identity(), |acc, &t| Affine1::compose(acc, t))
    }

    /// root <- frame through the batch's slot.
    fn resolved<const MEMOIZE: bool>(
        batch: &mut TransformBatchBase<Affine1, MEMOIZE>,
        frame: FrameId,
        ts: Timestamp,
    ) -> Affine1 {
        let slot = batch.request(frame, ts).unwrap();
        batch.response()[slot as usize]
    }

    /// root <- frame as memoized, without requesting it (no slot).
    fn memoized<const MEMOIZE: bool>(
        batch: &TransformBatchBase<Affine1, MEMOIZE>,
        frame: FrameId,
        ts: Timestamp,
    ) -> Option<Affine1> {
        batch.memo.get(&(frame, ts)).and_then(|result| result.ok())
    }

    #[test]
    fn endpoint_only_memoizes_just_the_requested_frame() {
        let t = tree();
        let mut batch = TransformBatchBase::<Affine1, false>::new(&t.store, t.map);
        resolved(&mut batch, t.laser, at(0));
        assert_eq!(memoized(&batch, t.base, at(0)), None);
        assert_eq!(memoized(&batch, t.odom, at(0)), None);
    }

    #[test]
    fn memoize_stores_every_intermediate_frame() {
        let t = tree();
        let mut batch = TransformBatchBase::<Affine1, true>::new(&t.store, t.map);
        resolved(&mut batch, t.laser, at(0));
        assert_eq!(
            memoized(&batch, t.base, at(0)),
            Some(compose(&[MAP_ODOM, ODOM_BASE]))
        );
        assert_eq!(memoized(&batch, t.odom, at(0)), Some(MAP_ODOM));
        // memoized but not requested: no slot
        assert_eq!(batch.response().len(), 1);
    }

    /// Every test below runs once per `MEMOIZE` variant; `TransformBatch` is shadowed by
    /// the variant under test.
    macro_rules! batch_tests {
        ($variant:ident, $memoize:literal) => {
            mod $variant {
                use super::*;
                type TransformBatch<'a> = TransformBatchBase<'a, Affine1, $memoize>;

                #[test]
                fn root_resolves_to_identity() {
                    let t = tree();
                    let mut at_top = TransformBatch::new(&t.store, t.map);
                    assert_eq!(resolved(&mut at_top, t.map, at(0)), Affine1::identity());
                    let mut mid_tree = TransformBatch::new(&t.store, t.base);
                    assert_eq!(resolved(&mut mid_tree, t.base, at(0)), Affine1::identity());
                }

                #[test]
                fn frame_below_root_composes_the_chain() {
                    let t = tree();
                    let mut batch = TransformBatch::new(&t.store, t.map);
                    let map_laser = resolved(&mut batch, t.laser, at(0));
                    assert_eq!(map_laser, compose(&[MAP_ODOM, ODOM_BASE, BASE_LASER]));
                    // child applied first: laser point -> base_link -> odom -> map
                    let x = 3.0;
                    assert_eq!(
                        map_laser.apply(x),
                        MAP_ODOM.apply(ODOM_BASE.apply(BASE_LASER.apply(x)))
                    );
                }

                #[test]
                fn root_mid_tree_uses_inverse_for_frames_above_and_beside() {
                    let t = tree();
                    let mut batch = TransformBatch::new(&t.store, t.base);
                    // below the root: plain chain
                    assert_eq!(resolved(&mut batch, t.laser, at(0)), BASE_LASER);
                    assert_eq!(resolved(&mut batch, t.camera, at(0)), BASE_CAMERA);
                    // above the root
                    assert_eq!(resolved(&mut batch, t.odom, at(0)), ODOM_BASE.inverse());
                    assert_eq!(
                        resolved(&mut batch, t.map, at(0)),
                        compose(&[ODOM_BASE.inverse(), MAP_ODOM.inverse()])
                    );
                    // beside the root, through the common ancestor odom
                    assert_eq!(
                        resolved(&mut batch, t.imu, at(0)),
                        compose(&[ODOM_BASE.inverse(), ODOM_IMU])
                    );
                }

                #[test]
                fn only_requested_frames_get_slots() {
                    let t = tree();
                    let mut batch = TransformBatch::new(&t.store, t.map);
                    let laser = batch.request(t.laser, at(0)).unwrap();
                    let camera = batch.request(t.camera, at(0)).unwrap();
                    assert_ne!(laser, camera);
                    // odom and base_link were resolved on the way but never requested
                    assert_eq!(batch.response().len(), 2);
                    assert_eq!(
                        batch.response()[laser as usize],
                        compose(&[MAP_ODOM, ODOM_BASE, BASE_LASER])
                    );
                    assert_eq!(
                        batch.response()[camera as usize],
                        compose(&[MAP_ODOM, ODOM_BASE, BASE_CAMERA])
                    );
                }

                #[test]
                fn same_frame_and_time_reuses_the_slot() {
                    let t = tree();
                    let mut batch = TransformBatch::new(&t.store, t.map);
                    let first = batch.request(t.laser, at(0)).unwrap();
                    let second = batch.request(t.laser, at(0)).unwrap();
                    assert_eq!(first, second);
                    assert_eq!(batch.response().len(), 1);
                }

                #[test]
                fn same_frame_at_different_times_gets_a_slot_per_time() {
                    let mut t = tree();
                    t.store
                        .insert(t.base, t.odom, Some(at(0)), Affine1::new(1.0, 0.0));
                    t.store
                        .insert(t.base, t.odom, Some(at(10)), Affine1::new(1.0, 10.0));
                    let mut batch = TransformBatch::new(&t.store, t.map);
                    let expected = |odom_base| compose(&[MAP_ODOM, odom_base]);
                    assert_eq!(
                        resolved(&mut batch, t.base, at(0)),
                        expected(Affine1::new(1.0, 0.0))
                    );
                    assert_eq!(
                        resolved(&mut batch, t.base, at(5)),
                        expected(Affine1::new(1.0, 5.0))
                    );
                    assert_eq!(
                        resolved(&mut batch, t.base, at(10)),
                        expected(Affine1::new(1.0, 10.0))
                    );
                    assert_eq!(batch.response().len(), 3);
                }

                #[test]
                fn disconnected_tree_is_no_path() {
                    let mut t = tree();
                    let world = t.store.intern("world");
                    let robot2 = t.store.intern("robot2");
                    t.store.insert(robot2, world, None, Affine1::new(2.0, 0.0));
                    let ghost = t.store.intern("ghost"); // interned, never in an edge

                    let mut batch = TransformBatch::new(&t.store, t.map);
                    assert!(matches!(
                        batch.request(robot2, at(0)),
                        Err(TransformError::NoPath)
                    ));
                    assert!(matches!(
                        batch.request(world, at(0)),
                        Err(TransformError::NoPath)
                    ));
                    assert!(matches!(
                        batch.request(ghost, at(0)),
                        Err(TransformError::NoPath)
                    ));
                    // errors take no slot, and the connected tree still resolves
                    assert_eq!(batch.response().len(), 0);
                    assert_eq!(
                        resolved(&mut batch, t.laser, at(0)),
                        compose(&[MAP_ODOM, ODOM_BASE, BASE_LASER])
                    );
                }

                #[test]
                fn no_sample_before_on_the_path_fails_and_repeats() {
                    let mut t = tree();
                    t.store.insert(t.base, t.odom, Some(at(10)), ODOM_BASE); // only a future sample
                    let mut batch = TransformBatch::new(&t.store, t.map);
                    for _ in 0..2 {
                        assert!(matches!(
                            batch.request(t.laser, at(5)),
                            Err(TransformError::NoSampleBefore { child }) if child == t.base
                        ));
                    }
                    assert_eq!(batch.response().len(), 0);
                    // a frame whose path avoids that edge is unaffected
                    assert_eq!(
                        resolved(&mut batch, t.imu, at(5)),
                        compose(&[MAP_ODOM, ODOM_IMU])
                    );
                    // and at the sample's time the same frame resolves
                    assert_eq!(
                        resolved(&mut batch, t.laser, at(10)),
                        compose(&[MAP_ODOM, ODOM_BASE, BASE_LASER])
                    );
                }

                #[test]
                fn unresolved_edge_above_the_common_ancestor_is_not_needed() {
                    let mut t = tree();
                    t.store.insert(t.odom, t.map, Some(at(10)), MAP_ODOM); // map <- odom unknown at 5
                    let mut batch = TransformBatch::new(&t.store, t.base);
                    // below the root and beside it (common ancestor odom): never touch map <- odom
                    assert_eq!(resolved(&mut batch, t.laser, at(5)), BASE_LASER);
                    assert_eq!(
                        resolved(&mut batch, t.imu, at(5)),
                        compose(&[ODOM_BASE.inverse(), ODOM_IMU])
                    );
                    // only map itself needs the missing edge; root-walk errors aren't reported
                    assert!(matches!(
                        batch.request(t.map, at(5)),
                        Err(TransformError::NoPath)
                    ));
                }

                #[test]
                fn cycle_entered_from_below_terminates() {
                    // the requested frame is not on the cycle, so only a visited check can stop the walk
                    let mut t = tree();
                    let a = t.store.intern("a");
                    let b = t.store.intern("b");
                    let below = t.store.intern("below_cycle");
                    t.store.insert(a, b, None, Affine1::identity());
                    t.store.insert(b, a, None, Affine1::identity());
                    t.store.insert(below, a, None, Affine1::identity());

                    let mut batch = TransformBatch::new(&t.store, t.map);
                    assert!(matches!(
                        batch.request(below, at(0)),
                        Err(TransformError::Cycle)
                    ));
                }

                #[test]
                fn cycle_is_detected_and_the_batch_keeps_working() {
                    let mut t = tree();
                    let a = t.store.intern("a");
                    let b = t.store.intern("b");
                    let below = t.store.intern("below_cycle");
                    let selfie = t.store.intern("self_parent");
                    t.store.insert(a, b, None, Affine1::identity());
                    t.store.insert(b, a, None, Affine1::identity());
                    t.store.insert(below, a, None, Affine1::identity());
                    t.store.insert(selfie, selfie, None, Affine1::identity());

                    let mut batch = TransformBatch::new(&t.store, t.map);
                    for frame in [a, b, below, selfie] {
                        assert!(
                            matches!(batch.request(frame, at(0)), Err(TransformError::Cycle)),
                            "{}",
                            t.store.name(frame)
                        );
                    }
                    assert_eq!(batch.response().len(), 0);
                    assert_eq!(
                        resolved(&mut batch, t.laser, at(0)),
                        compose(&[MAP_ODOM, ODOM_BASE, BASE_LASER])
                    );
                }
            }
        };
    }

    batch_tests!(endpoint_only, false);
    batch_tests!(memoize_intermediates, true);
}
