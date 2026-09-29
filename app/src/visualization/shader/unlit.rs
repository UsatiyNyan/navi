use crate::visualization::{instance, texture, transform_frame, vertex};
use lib::render;
use wgpu::{self, util::DeviceExt};

pub struct Unlit {
    render_pipeline: wgpu::RenderPipeline,

    transform_frames_buffer: wgpu::Buffer,
    transform_frames_bind_group: wgpu::BindGroup,

    mvp_buffer: wgpu::Buffer,
    mvp_bind_group: wgpu::BindGroup,
}

impl Unlit {
    pub fn new(
        device: &wgpu::Device,
        handle: &render::Handle,
        transform_frames_init: &[transform_frame::TransformFrame],
        mvp_init: glam::Mat4,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("unlit.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./unlit.wgsl").into()),
        });

        // vvv transform_frames
        let transform_frames_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Frames Buffer"),
                contents: bytemuck::cast_slice(transform_frames_init),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let transform_frames_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Transform Frames Bind Group Layout"),
            });
        let transform_frames_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &transform_frames_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_frames_buffer.as_entire_binding(),
            }],
            label: Some("Transform Frames Bind Group"),
        });
        // ^^^ transform_frames

        // vvv mvp
        let mvp_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("MVP Buffer"),
            contents: bytemuck::cast_slice(&[mvp_init]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let mvp_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("MVP Bind Group Layout"),
            });
        let mvp_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mvp_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mvp_buffer.as_entire_binding(),
            }],
            label: Some("MVP Bind Group"),
        });
        // ^^^ mvp

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&transform_frames_bind_group_layout),
                    Some(&mvp_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(vertex::Vertex::desc()),
                    Some(instance::Instance::desc()),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: handle.config().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        Self {
            render_pipeline,
            transform_frames_buffer,
            transform_frames_bind_group,
            mvp_buffer,
            mvp_bind_group,
        }
    }

    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.render_pipeline
    }

    pub fn transform_frames_bind_group(&self) -> &wgpu::BindGroup {
        &self.transform_frames_bind_group
    }

    pub fn mvp_bind_group(&self) -> &wgpu::BindGroup {
        &self.mvp_bind_group
    }
}
