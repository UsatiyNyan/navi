#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTransform {
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
    pub padding0: u32,
}

impl From<&Transform> for GpuTransform {
    fn from(value: &Transform) -> Self {
        Self {
            rotation: value.rotation.to_array().map(|x| x as f32),
            translation: value.translation.to_array().map(|x| x as f32),
            padding0: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    rotation: glam::DQuat,
    translation: glam::DVec3,
}

impl lib::buffer::Transform for Transform {
    fn identity() -> Self {
        Self {
            rotation: glam::DQuat::IDENTITY,
            translation: glam::DVec3::ZERO,
        }
    }
    // parent <- child * child <- x = parent <- x
    fn compose(parent: Self, child: Self) -> Self {
        Self {
            rotation: (parent.rotation * child.rotation).normalize(),
            translation: (parent.rotation * child.translation) + parent.translation,
        }
    }
    fn inverse(self) -> Self {
        // conjugate same as inverse for unit quats, which they are, see normalize
        Self {
            rotation: self.rotation.conjugate(),
            translation: -(self.rotation.conjugate() * self.translation),
        }
    }
    // t in [0, 1]; t = 0 -> a, t = 1 -> b
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self {
            rotation: a.rotation.lerp(b.rotation, t),
            translation: a.translation.lerp(b.translation, t),
        }
    }
}
