use super::vertex;
use lib::buffer;
use std::ops::Range;
use wgpu::{self, util::DeviceExt};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MeshId(u32);

impl From<MeshId> for u32 {
    fn from(value: MeshId) -> Self {
        value.0
    }
}

impl From<u32> for MeshId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

pub struct MeshStore {
    names: buffer::Interner<MeshId>,
    items: Vec<Mesh>,
}

impl Default for MeshStore {
    fn default() -> Self {
        Self {
            names: Default::default(),
            items: Default::default(),
        }
    }
}

impl MeshStore {
    // name has to be unique
    pub fn entry<'a>(&'a mut self, label: &'a str) -> MeshEntry<'a> {
        match self.lookup(label) {
            Some(id) => MeshEntry::Occupied(id),
            None => MeshEntry::Vacant(VacantMeshEntry(self, label)),
        }
    }
    pub fn get(&self, id: MeshId) -> &Mesh {
        &self.items[Into::<u32>::into(id) as usize]
    }
    pub fn lookup(&self, label: &str) -> Option<MeshId> {
        self.names.lookup(label)
    }
    pub fn label(&self, id: MeshId) -> &str {
        self.names.name(id)
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub enum MeshEntry<'a> {
    Occupied(MeshId),
    Vacant(VacantMeshEntry<'a>),
}

pub struct VacantMeshEntry<'a>(&'a mut MeshStore, &'a str);
impl<'a> VacantMeshEntry<'a> {
    pub fn insert<V: vertex::Vertex>(
        self,
        device: &wgpu::Device,
        vertices: &[V],
        indices: &[u32],
    ) -> MeshId {
        let (store, label) = (self.0, self.1);
        debug_assert_eq!(store.names.len(), store.items.len());
        let id = store.names.intern(label);
        debug_assert_eq!(Into::<u32>::into(id) as usize, store.items.len());
        let mesh = Mesh::new(device, label, vertices, indices);
        store.items.push(mesh);
        id
    }
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    // TODO: material
}

impl Mesh {
    fn new<V: vertex::Vertex>(
        device: &wgpu::Device,
        label: &str,
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
