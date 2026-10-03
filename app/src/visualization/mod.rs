mod camera;
mod instance;
mod mesh;
mod shader;
mod texture;
mod transform_frame;
mod vertex;

use super::app;
use lib::{buffer, render};
use mesh::DrawMesh;
use wgpu::{self, util::DeviceExt};
use shader::UseShader;

struct Scene {
    mesh: mesh::Mesh,
    instance_buffer: wgpu::Buffer,
}

pub struct Visualization {
    shader: shader::Unlit,
    scene: Scene,

    depth_texture: texture::Texture,

    transform_frames: Vec<transform_frame::TransformFrame>,
    camera: camera::Camera,
}

impl Visualization {
    pub fn new(handle: &render::Handle) -> Self {
        let device = handle.device();
        let config = handle.config();

        let transform_frames = vec![transform_frame::TransformFrame {
            rotation: (0.0, 0.0, 0.0, 1.0).into(),
            translation: (0.0, 0.0, 0.0).into(),
            padding0: 0,
        }];
        let camera = camera::Camera {
            eye: (1.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: glam::Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let shader = shader::Unlit::new(device, handle, &transform_frames, camera.mvp());

        #[rustfmt::skip]
        const VERTICES: &[vertex::UnlitVertex] = 
        {
            let r = [1.0, 0.0, 0.0];
            let g = [0.0, 1.0, 0.0];
            let b = [0.0, 0.0, 1.0];
            type V = vertex::UnlitVertex;
            &[
                V{ position: [ 0.5, 0.5, 0.5, ], color: r}, // top right
                V{ position: [ 0.5, -0.5, 0.5, ], color: g }, // bottom right
                V{ position: [ -0.5, -0.5, 0.5, ], color: r }, // bottom left
                V{ position: [ -0.5, 0.5, 0.5, ], color: b }, // top left
                // right face
                V{ position: [ 0.5, 0.5, 0.5, ], color: g }, // top right
                V{ position: [ 0.5, -0.5, 0.5, ], color: r }, // bottom right
                V{ position: [ 0.5, -0.5, -0.5, ], color: g }, // bottom left
                V{ position: [ 0.5, 0.5, -0.5, ], color: b }, // top left
                // back face
                V{ position: [ 0.5, 0.5, -0.5, ], color: b }, // top right
                V{ position: [ 0.5, -0.5, -0.5, ], color: g }, // bottom right
                V{ position: [ -0.5, -0.5, -0.5, ], color: b }, // bottom left
                V{ position: [ -0.5, 0.5, -0.5, ], color: r }, // top left
                // left face
                V{ position: [ -0.5, 0.5, -0.5, ], color: r }, // top right
                V{ position: [ -0.5, -0.5, -0.5, ], color: g }, // bottom right
                V{ position: [ -0.5, -0.5, 0.5, ], color: r }, // bottom left
                V{ position: [ -0.5, 0.5, 0.5, ], color: b }, // top left
                // top face
                V{ position: [ 0.5, 0.5, 0.5, ], color: g }, // top right
                V{ position: [ 0.5, 0.5, -0.5, ], color: r }, // bottom right
                V{ position: [ -0.5, 0.5, -0.5, ], color: g }, // bottom left
                V{ position: [ -0.5, 0.5, 0.5, ], color: b }, // top left
                // bottom face
                V{ position: [ 0.5, -0.5, 0.5, ], color: b }, // top right
                V{ position: [ 0.5, -0.5, -0.5, ], color: g }, // bottom right
                V{ position: [ -0.5, -0.5, -0.5, ], color: b }, // bottom left
                V{ position: [ -0.5, -0.5, 0.5, ], color: r }, // top left
            ]
        };
        const INDICES: &[u32] = &[
            3,  1,  0,   3,  2,  1, // front face
            4,  5,  7,   5,  6,  7, // right face
            8,  9, 11,   9, 10, 11, // back face
            12, 13, 15,  13, 14, 15, // left face
            16, 17, 19,  17, 18, 19, // top face
            23, 21, 20,  23, 22, 21, // bottom face
        ];
        const INSTANCES: &[instance::Instance] = &[instance::Instance {
            translation: [0.0, 0.0, 0.0],
            frame: 0,
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }];

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(INSTANCES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let scene = Scene {
            mesh: mesh::Mesh::new(device, "cube".into(), VERTICES, INDICES),
            instance_buffer,
        };

        let depth_texture = texture::Texture::new_depth_texture(&device, config, "Depth Texture");

        Self {
            shader,
            scene,
            depth_texture,
            transform_frames,
            camera,
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
            render_pass.set_shader(&self.shader);
            render_pass.set_vertex_buffer(1, self.scene.instance_buffer.slice(..));
            render_pass.draw_mesh_instanced(&self.scene.mesh, 0..1);
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
