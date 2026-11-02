use std::marker::PhantomData;
use wgpu;

pub struct GpuBuffer<T> {
    label: String,
    buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages, // COPY_DST always added
    capacity: u32,
    len: u32,
    _type: PhantomData<T>,
}

impl<T: bytemuck::Pod> GpuBuffer<T> {
    pub fn new(device: &wgpu::Device, label: &str, usage: wgpu::BufferUsages) -> Self {
        Self::with_capacity(device, label, usage, 1)
    }
    pub fn with_capacity(
        device: &wgpu::Device,
        label: &str,
        usage: wgpu::BufferUsages,
        capacity: u32,
    ) -> Self {
        const { assert!(std::mem::size_of::<T>() % 4 == 0) };
        let buffer = Self::create_buffer(device, label, capacity as usize, usage);
        Self {
            label: label.into(),
            buffer,
            usage,
            capacity,
            len: 0,
            _type: Default::default(),
        }
    }
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) -> bool {
        let need_realloc = data.len() > self.capacity as usize;
        if need_realloc {
            self.capacity = data.len().next_power_of_two() as u32;
            self.buffer =
                Self::create_buffer(device, &self.label, self.capacity as usize, self.usage);
        }
        self.len = data.len() as u32;
        if !data.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
        }
        need_realloc
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        let upper = (self.len as usize * std::mem::size_of::<T>()) as u64;
        self.buffer.slice(..upper)
    }
    pub fn resource(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    fn create_buffer(
        device: &wgpu::Device,
        label: &str,
        capacity: usize,
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&label),
            size: (capacity * size_of::<T>()) as wgpu::BufferAddress,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

pub struct GpuBinding {
    label: String,
    binding: u32,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GpuBinding {
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        binding: u32,
        resource: wgpu::BindingResource,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(&format!("{label} Layout")),
        });
        let bind_group =
            Self::create_bind_group(device, label, binding, resource, &bind_group_layout);
        Self {
            label: label.into(),
            binding,
            bind_group,
            bind_group_layout,
        }
    }
    fn create_bind_group(
        device: &wgpu::Device,
        label: &str,
        binding: u32,
        resource: wgpu::BindingResource,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding, resource }],
            label: Some(&format!("{label} Group")),
        })
    }
    pub fn rebind(&mut self, device: &wgpu::Device, resource: wgpu::BindingResource) {
        self.bind_group = Self::create_bind_group(
            device,
            &self.label,
            self.binding,
            resource,
            &self.bind_group_layout,
        );
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

pub struct GpuBufferWithBinding<T> {
    buffer: GpuBuffer<T>,
    binding: GpuBinding,
}

impl<T: bytemuck::Pod> GpuBufferWithBinding<T> {
    pub fn new(device: &wgpu::Device, label: &str, binding: u32) -> Self {
        Self::with_capacity(device, label, binding, 1)
    }
    pub fn with_capacity(device: &wgpu::Device, label: &str, binding: u32, capacity: u32) -> Self {
        let buffer = GpuBuffer::with_capacity(
            device,
            &format!("{} Buffer", label),
            wgpu::BufferUsages::STORAGE,
            capacity,
        );
        let binding = GpuBinding::new(
            device,
            &format!("{} Binding", label),
            binding,
            buffer.resource(),
        );
        Self { buffer, binding }
    }
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        if self.buffer.write(device, queue, data) {
            self.binding.rebind(device, self.buffer.resource());
        }
    }
    pub fn buffer(&self) -> &GpuBuffer<T> {
        &self.buffer
    }
    pub fn binding(&self) -> &GpuBinding {
        &self.binding
    }
}
