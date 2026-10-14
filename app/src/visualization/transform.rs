use lib::buffer;
use std::collections::{HashMap, HashSet};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformFrame {
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
    pub padding0: u32,
}

#[derive(Debug, Clone)]
pub struct TransformFrameRecord {
    parent: String,
    child: String,
    rotation: glam::Quat,
    translation: glam::Vec3,
}

pub struct TransformTreeCompletion {
    requested: HashSet<String>,
    orphans: HashMap<String, Vec<(String, glam::Quat, glam::Vec3)>>,
    from_root: HashMap<String, (glam::Quat, glam::Vec3)>,
}

impl TransformTreeCompletion {
    pub fn new(root_frame: String, requested: HashSet<String>) -> Self {
        Self {
            requested,
            orphans: Default::default(),
            from_root: HashMap::from([(root_frame, (glam::Quat::IDENTITY, glam::Vec3::ZERO))]),
        }
    }

    pub fn insert_from_root(
        &mut self,
        child_frame: String,
        child_transform: (glam::Quat, glam::Vec3),
    ) {
        self.requested.remove(&child_frame);
        let (child_rotation, child_translation) = child_transform;
        if let Some(orphans) = self.orphans.remove(&child_frame) {
            for (orphan_frame, orphan_rotation, orphan_translation) in orphans {
                let orphan_transform = combine_transforms(
                    &child_rotation,
                    &child_translation,
                    &orphan_rotation,
                    &orphan_translation,
                );
                self.insert_from_root(orphan_frame, orphan_transform);
            }
        }
        match self.from_root.entry(child_frame) {
            std::collections::hash_map::Entry::Occupied(_) => {}
            std::collections::hash_map::Entry::Vacant(entry) => {
                let _ = entry.insert(child_transform);
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.requested.is_empty()
    }

    pub(crate) fn calc_frames(&self) -> Box<[TransformFrame]> {
        todo!()
    }
}

impl buffer::Record<TransformTreeCompletion> for TransformFrameRecord {
    fn accum(&self, completion: &mut TransformTreeCompletion) {
        if let Some((parent_rotation, parent_translation)) = completion.from_root.get(&self.parent)
        {
            let (child_rotation, child_translation) = combine_transforms(
                parent_rotation,
                parent_translation,
                &self.rotation,
                &self.translation,
            );
            completion.insert_from_root(self.child.clone(), (child_rotation, child_translation));
        } else {
            completion
                .orphans
                .entry(self.parent.clone())
                .or_default()
                .push((self.child.clone(), self.rotation, self.translation));
        }
    }
}

fn combine_transforms(
    parent_rotation: &glam::Quat,
    parent_translation: &glam::Vec3,
    child_rotation: &glam::Quat,
    child_translation: &glam::Vec3,
) -> (glam::Quat, glam::Vec3) {
    (
        (parent_rotation * child_rotation).normalize(),
        transform_point(parent_rotation, parent_translation, child_translation),
    )
}

fn transform_point(
    rotation: &glam::Quat,
    translation: &glam::Vec3,
    point: &glam::Vec3,
) -> glam::Vec3 {
    (rotation * point) + translation
}
