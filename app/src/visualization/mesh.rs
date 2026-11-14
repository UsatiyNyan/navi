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
    cpu_queue: Vec<(MeshId, CpuMesh)>,
    gpu_items: Vec<Option<GpuMesh>>,
}

impl MeshStore {
    // name has to be unique
    pub fn entry<'a>(&'a mut self, label: &'a str) -> MeshEntry<'a> {
        match self.lookup(label) {
            Some(id) => MeshEntry::Occupied(OccupiedMeshEntry { store: self, id }),
            None => MeshEntry::Vacant(VacantMeshEntry { store: self, label }),
        }
    }
    pub fn get(&self, id: MeshId) -> Option<&GpuMesh> {
        self.gpu_items
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
        self.gpu_items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.gpu_items.is_empty()
    }

    fn insert_impl(&mut self, label: &str, value: CpuMesh) -> MeshId {
        let id = self.reserve_impl(label);
        self.cpu_queue.push((id, value));
        id
    }
    fn reserve_impl(&mut self, label: &str) -> MeshId {
        debug_assert_eq!(self.names.len(), self.gpu_items.len());
        let id = self.names.intern(label);
        debug_assert_eq!(Into::<u32>::into(id) as usize, self.gpu_items.len());
        self.gpu_items.push(None);
        id
    }
    fn assign_impl(&mut self, id: MeshId, value: CpuMesh) -> Option<GpuMesh> {
        let prev = self.reset_impl(id);
        self.cpu_queue.push((id, value));
        prev
    }
    fn reset_impl(&mut self, id: MeshId) -> Option<GpuMesh> {
        let index = Into::<u32>::into(id) as usize;
        debug_assert!(index < self.gpu_items.len());
        std::mem::replace(&mut self.gpu_items[index], None)
    }

    pub fn upload(&mut self, device: &wgpu::Device) {
        for (id, cpu_mesh) in &self.cpu_queue {
            let label = self.label(*id);
            let gpu_mesh = match &cpu_mesh.vertices {
                CpuVertices::Unlit(vertices) => {
                    GpuMesh::new(device, label, &vertices, &cpu_mesh.indices)
                }
            };
            let index = Into::<u32>::into(*id) as usize;
            debug_assert!(index < self.gpu_items.len());
            self.gpu_items[index] = Some(gpu_mesh);
        }
        self.cpu_queue.clear();
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
    pub fn value(&self) -> Option<&GpuMesh> {
        self.store.get(self.id)
    }

    pub fn assign(self, value: CpuMesh) -> Option<GpuMesh> {
        self.store.assign_impl(self.id, value)
    }
    pub fn reset(self) -> Option<GpuMesh> {
        self.store.reset_impl(self.id)
    }
}

pub struct VacantMeshEntry<'a> {
    store: &'a mut MeshStore,
    label: &'a str,
}
impl<'a> VacantMeshEntry<'a> {
    pub fn insert(self, value: CpuMesh) -> MeshId {
        self.store.insert_impl(self.label, value)
    }
    pub fn reserve(self) -> MeshId {
        self.store.reserve_impl(self.label)
    }
}

pub struct CpuMesh {
    pub vertices: CpuVertices,
    pub indices: Box<[u32]>,
}
impl CpuMesh {
    pub fn unlit(vertices: &[vertex::UnlitVertex], indices: &[u32]) -> Self {
        Self {
            vertices: CpuVertices::Unlit(vertices.into()),
            indices: indices.into(),
        }
    }
}

pub enum CpuVertices {
    Unlit(Box<[vertex::UnlitVertex]>),
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    // TODO: material
}

impl GpuMesh {
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
    fn draw_mesh(&mut self, mesh: &GpuMesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(&mut self, mesh: &GpuMesh, instances: Range<u32>);
}

impl<'a> DrawMesh for wgpu::RenderPass<'a> {
    fn draw_mesh_instanced(&mut self, mesh: &GpuMesh, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.index_count, 0, instances);
    }
}
