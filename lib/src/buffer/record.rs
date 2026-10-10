use std::marker::PhantomData;

pub trait Record<C>: Clone {
    fn accum(&self, completion: &mut C);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    Snapshot,
    Incremental,
}

impl RecordType {
    pub fn is_snapshot(&self) -> bool {
        *self == RecordType::Snapshot
    }
    pub fn is_incremental(&self) -> bool {
        *self == RecordType::Incremental
    }
}

#[derive(Debug, Clone)]
pub struct RecordNode<C, T: Record<C>> {
    pub record_value: T,
    pub record_type: RecordType,
    _completion: PhantomData<C>,
}

impl<C, T: Record<C>> RecordNode<C, T> {
    pub fn new(record_value: T, record_type: RecordType) -> Self {
        Self {
            record_value,
            record_type,
            _completion: PhantomData {},
        }
    }
}
