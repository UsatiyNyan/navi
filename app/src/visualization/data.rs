use lib::buffer;

pub enum Topic {
    Pose(buffer::Timeline<Pose>),
    Mesh(buffer::Timeline<Mesh>),
    PointCloud(buffer::Timeline<PointCloud>),
    // TODO: Instance + InstanceMetadata
}

impl buffer::TopicInterface for Topic {
    fn evict_around(&mut self, ts: buffer::Timestamp, window: buffer::Duration, margin: usize) {
        match self {
            Topic::Pose(timeline) => timeline.evict_around(ts, window, margin),
            Topic::Mesh(timeline) => timeline.evict_around(ts, window, margin),
            Topic::PointCloud(timeline) => timeline.evict_around(ts, window, margin),
        }
    }
}

pub struct Pose {
    pub frame_id: buffer::FrameId,
    pub rotation: glam::DQuat,
    pub position: glam::DVec3,
}

pub struct Mesh {
    pub frame_id: buffer::FrameId,
    pub positions: Box<[glam::DVec4]>,
    pub colors: Box<[glam::Vec4]>,
    pub indices: Box<[u32]>,
}

pub struct PointCloud {
    pub frame_id: buffer::FrameId,
    pub points: Box<[glam::DVec4]>,
    pub colors: Box<[glam::Vec4]>,
}
