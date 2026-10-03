use super::vertex;
use std::ops::Range;
use wgpu::{self, util::DeviceExt};

pub struct Mesh {
    pub label: String, // TODO: this smells
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    // TODO: material
}

impl Mesh {
    pub fn new<V: vertex::Vertex>(
        device: &wgpu::Device,
        label: String,
        vertices: &[V],
        indices: &[u32],
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&(format!("{}: VB", label))),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&(format!("{}: IB", label))),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            label,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}

pub trait DrawMesh {
    fn draw_mesh(&mut self, mesh: &Mesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(&mut self, mesh: &Mesh, instances: Range<u32>);
}

impl<'a> DrawMesh for wgpu::RenderPass<'a> {
    fn draw_mesh_instanced(&mut self, mesh: &Mesh, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.index_count, 0, instances);
    }
}
