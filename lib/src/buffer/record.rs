pub trait Record<T: Clone> {
    fn merge(&mut self, other: T);
    fn combine(&self, other: &T) -> T;
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
    pub fn combine(&self, other: &Self) -> Self {
        match (self, other) {
            (RecordType::Snapshot, RecordType::Snapshot) => RecordType::Snapshot,
            (_, _) => RecordType::Incremental,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordNode<T: Record<T> + Clone> {
    pub record_value: T,
    pub record_type: RecordType,
}

impl<T: Record<T> + Clone> RecordNode<T> {
    pub fn new(record_value: T, record_type: RecordType) -> Self {
        Self {
            record_value,
            record_type,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.record_value.merge(other.record_value);
        self.record_type = self.record_type.combine(&other.record_type);
    }

    pub fn combine(&self, other: &Self) -> Self {
        let combined_value = self.record_value.combine(&other.record_value);
        let combined_type = self.record_type.combine(&other.record_type);
        Self::new(combined_value, combined_type)
    }
}
