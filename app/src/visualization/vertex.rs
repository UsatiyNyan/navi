pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

// @location(0) position: vec3<f32>,
// @location(1) color: vec3<f32>,
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UnlitVertex {
    pub position: [f32; 3],
    pub padding0: u32,
    pub color: [f32; 4],
}

impl UnlitVertex {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position,
            padding0: 0,
            color,
        }
    }
}

impl Vertex for UnlitVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<UnlitVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}
