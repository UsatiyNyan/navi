mod instance;
mod shader;
mod texture;
mod vertex;

use super::app;
use lib::{buffer, render};
use wgpu::{self, util::DeviceExt};

pub struct Visualization {
    shader: shader::Unlit,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    depth_texture: texture::Texture,
}

impl Visualization {
    pub fn new(handle: &render::Handle) -> Self {
        let device = handle.device();
        let shader = shader::Unlit::new(device, handle);

        #[rustfmt::skip]
        const VERTICES: &[vertex::Vertex] = &[
            vertex::Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
            vertex::Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
            vertex::Vertex { position: [0.0, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
        ];
        const INDICES: &[u16] = &[0, 1, 2];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let depth_texture =
            texture::Texture::new_depth_texture(&device, handle.config(), "Depth Texture");

        Self {
            shader,
            vertex_buffer,
            index_buffer,
            depth_texture,
        }
    }

    pub fn render(
        &self,
        model: &app::Model,
        render_handle: &mut render::Handle,
        buffer_state: &mut buffer::State<app::Record>,
    ) -> anyhow::Result<()> {
        let mut frame = match render_handle.begin_frame() {
            Ok(frame) => frame,
            Err(error) => match error {
                render::BeginFrameError::Skip => {
                    return Ok(());
                }
                render::BeginFrameError::Retry => {
                    render_handle.request_redraw();
                    return Ok(());
                }
                render::BeginFrameError::Lost => anyhow::bail!("Lost render handle"),
            },
        };

        {
            let mut render_pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
            render_pass.set_pipeline(self.shader.render_pipeline());
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            let indices_count = (self.index_buffer.size() / 2) as u32;
            render_pass.draw_indexed(0..indices_count, 0, 0..1);
        }

        render_handle.end_frame(frame.surface, frame.encoder);
        Ok(())
    }

    // TODO: this smells
    pub fn resize(&mut self, handle: &render::Handle) {
        self.depth_texture =
            texture::Texture::new_depth_texture(handle.device(), handle.config(), "Depth Texture");
    }
}
