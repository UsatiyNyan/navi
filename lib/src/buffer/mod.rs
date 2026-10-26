mod intern;
mod timeline;
mod topic;
mod transform;

pub use intern::*;
pub use timeline::*;
pub use topic::*;
pub use transform::*;

pub struct Buffer<D, T> {
    pub topics: TopicStore<D>,
    pub tf: TransformStore<T>,
}

impl<D, T> Default for Buffer<D, T> {
    fn default() -> Self {
        Self {
            topics: Default::default(),
            tf: Default::default(),
        }
    }
}
