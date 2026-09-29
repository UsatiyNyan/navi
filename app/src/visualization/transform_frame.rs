#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformFrame {
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
    pub padding0: u32,
}
