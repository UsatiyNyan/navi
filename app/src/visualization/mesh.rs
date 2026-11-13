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

#[derive(Default)]
pub struct MeshStore {
    names: buffer::Interner<MeshId>,
    items: Vec<Option<Mesh>>,
}

impl MeshStore {
    // name has to be unique
    pub fn entry<'a>(&'a mut self, label: &'a str) -> MeshEntry<'a> {
        match self.lookup(label) {
            Some(id) => MeshEntry::Occupied(OccupiedMeshEntry { store: self, id }),
            None => MeshEntry::Vacant(VacantMeshEntry { store: self, label }),
        }
    }
    pub fn get(&self, id: MeshId) -> Option<&Mesh> {
        self.items
            .get(Into::<u32>::into(id) as usize)
            .and_then(Option::as_ref)
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

    fn insert_impl(&mut self, label: &str, value: Option<Mesh>) -> MeshId {
        debug_assert_eq!(self.names.len(), self.items.len());
        let id = self.names.intern(label);
        debug_assert_eq!(Into::<u32>::into(id) as usize, self.items.len());
        self.items.push(value);
        id
    }

    fn assign_impl(&mut self, id: MeshId, value: Option<Mesh>) -> Option<Mesh> {
        let index = Into::<u32>::into(id) as usize;
        debug_assert!(index < self.items.len());
        std::mem::replace(&mut self.items[index], value)
    }
}

pub enum MeshEntry<'a> {
    Occupied(OccupiedMeshEntry<'a>),
    Vacant(VacantMeshEntry<'a>),
}

pub struct OccupiedMeshEntry<'a> {
    store: &'a mut MeshStore,
    id: MeshId,
}
impl<'a> OccupiedMeshEntry<'a> {
    pub fn id(&self) -> MeshId {
        self.id
    }

    pub fn value(&self) -> Option<&Mesh> {
        self.store.get(self.id)
    }

    pub fn assign<V: vertex::Vertex>(
        self,
        device: &wgpu::Device,
        vertices: &[V],
        indices: &[u32],
    ) -> Option<Mesh> {
        let label = self.store.label(self.id);
        self.store
            .assign_impl(self.id, Some(Mesh::new(device, label, vertices, indices)))
    }
}

pub struct VacantMeshEntry<'a> {
    store: &'a mut MeshStore,
    label: &'a str,
}
impl<'a> VacantMeshEntry<'a> {
    pub fn insert<V: vertex::Vertex>(
        self,
        device: &wgpu::Device,
        vertices: &[V],
        indices: &[u32],
    ) -> MeshId {
        self.store.insert_impl(
            self.label,
            Some(Mesh::new(device, self.label, vertices, indices)),
        )
    }

    pub fn reserve(self) -> MeshId {
        self.store.insert_impl(self.label, None)
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
