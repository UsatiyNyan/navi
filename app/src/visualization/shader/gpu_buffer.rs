use std::marker::PhantomData;
use wgpu;

pub struct GpuBuffer<T> {
    label: String,
    binding: u32,
    buffer: wgpu::Buffer,
    capacity: usize,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    _type: PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> GpuBuffer<T> {
    pub fn new(device: &wgpu::Device, label: String, binding: u32) -> Self {
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
            label: Some(&format!("{label} Bind Group Layout")),
        });

        let capacity = 1;
        let (buffer, bind_group) = Self::create_buffer_with_bind_group(
            device,
            &label,
            binding,
            &bind_group_layout,
            capacity,
        );

        Self {
            label,
            binding,
            buffer,
            capacity,
            bind_group,
            bind_group_layout,
            _type: PhantomData {},
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn write_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        if data.len() > self.capacity {
            self.capacity = data.len().next_power_of_two();
            (self.buffer, self.bind_group) = Self::create_buffer_with_bind_group(
                device,
                &self.label,
                self.binding,
                &self.bind_group_layout,
                self.capacity,
            );
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    fn create_buffer_with_bind_group(
        device: &wgpu::Device,
        label: &String,
        binding: u32,
        bind_group_layout: &wgpu::BindGroupLayout,
        capacity: usize,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{label} Buffer")),
            size: (capacity * size_of::<T>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&format!("{label} Bind Group")),
        });
        (buffer, bind_group)
    }
}
