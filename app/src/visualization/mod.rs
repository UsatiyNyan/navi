mod camera;
pub(crate) mod data;
mod instance;
pub(crate) mod mesh;
mod shader;
mod texture;
pub(crate) mod transform;
mod vertex;

use super::app;
use lib::render;
use mesh::DrawMesh;
use shader::UseShader;
use wgpu::{self, util::DeviceExt};

struct Scene {
    mesh_id: mesh::MeshId,
    instance_buffer: wgpu::Buffer,
}

pub struct Visualization {
    shader: shader::Unlit,
    scene: Scene,

    depth_texture: texture::Texture,

    camera: camera::Camera,
}

impl Visualization {
    pub fn new(handle: &render::Handle, resources: &mut app::Resources) -> Self {
        let device = handle.device();
        let config = handle.config();

        let camera = camera::Camera {
            eye: (1.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: glam::Vec3::Y,
            aspect: aspect_from_config(config),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let shader = shader::Unlit::new(device, handle, camera.mvp());

        let vertices: &[vertex::UnlitVertex] = {
            let r = [1.0, 0.0, 0.0, 1.0];
            let g = [0.0, 1.0, 0.0, 1.0];
            let b = [0.0, 0.0, 1.0, 1.0];
            type V = vertex::UnlitVertex;
            &[
                V::new([0.5, 0.5, 0.5], r),   // top right
                V::new([0.5, -0.5, 0.5], g),  // bottom right
                V::new([-0.5, -0.5, 0.5], r), // bottom left
                V::new([-0.5, 0.5, 0.5], b),  // top left
                // right face
                V::new([0.5, 0.5, 0.5], g),   // top right
                V::new([0.5, -0.5, 0.5], r),  // bottom right
                V::new([0.5, -0.5, -0.5], g), // bottom left
                V::new([0.5, 0.5, -0.5], b),  // top left
                // back face
                V::new([0.5, 0.5, -0.5], b),   // top right
                V::new([0.5, -0.5, -0.5], g),  // bottom right
                V::new([-0.5, -0.5, -0.5], b), // bottom left
                V::new([-0.5, 0.5, -0.5], r),  // top left
                // left face
                V::new([-0.5, 0.5, -0.5], r),  // top right
                V::new([-0.5, -0.5, -0.5], g), // bottom right
                V::new([-0.5, -0.5, 0.5], r),  // bottom left
                V::new([-0.5, 0.5, 0.5], b),   // top left
                // top face
                V::new([0.5, 0.5, 0.5], g),   // top right
                V::new([0.5, 0.5, -0.5], r),  // bottom right
                V::new([-0.5, 0.5, -0.5], g), // bottom left
                V::new([-0.5, 0.5, 0.5], b),  // top left
                // bottom face
                V::new([0.5, -0.5, 0.5], b),   // top right
                V::new([0.5, -0.5, -0.5], g),  // bottom right
                V::new([-0.5, -0.5, -0.5], b), // bottom left
                V::new([-0.5, -0.5, 0.5], r),  // top left
            ]
        };
        const INDICES: &[u32] = &[
            3, 1, 0, 3, 2, 1, // front face
            4, 5, 7, 5, 6, 7, // right face
            8, 9, 11, 9, 10, 11, // back face
            12, 13, 15, 13, 14, 15, // left face
            16, 17, 19, 17, 18, 19, // top face
            23, 21, 20, 23, 22, 21, // bottom face
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

        let mesh_id = match resources.mesh_store.entry("cube") {
            mesh::MeshEntry::Occupied(entry) => entry.id(),
            mesh::MeshEntry::Vacant(entry) => entry.insert(mesh::CpuMesh::unlit(vertices, INDICES)),
        };

        let scene = Scene {
            mesh_id,
            instance_buffer,
        };

        let depth_texture = texture::Texture::new_depth_texture(device, config, "Depth Texture");

        Self {
            shader,
            scene,
            depth_texture,
            camera,
        }
    }

    pub fn render(
        &mut self,
        model: &app::Model,
        render_handle: &mut render::Handle,
        resources: &mut app::Resources,
    ) -> anyhow::Result<()> {
        resources.mesh_store.upload(render_handle.device());

        const TRANSFORM_FRAMES: &[transform::GpuTransform] = &[transform::GpuTransform {
            rotation: [0.0, 0.0, 0.0, 1.0],
            translation: [0.0, 0.0, 0.0],
            padding0: 0,
        }];
        self.shader.set_transform_frames_buffer(
            render_handle.device(),
            render_handle.queue(),
            TRANSFORM_FRAMES,
        );

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

            if let Some(mesh) = resources.mesh_store.get(self.scene.mesh_id) {
                render_pass.draw_mesh_instanced(mesh, 0..1);
            }
        }

        render_handle.end_frame(frame.surface, frame.encoder);
        Ok(())
    }

    // TODO: this smells
    pub fn resize(&mut self, handle: &render::Handle) {
        self.depth_texture =
            texture::Texture::new_depth_texture(handle.device(), handle.config(), "Depth Texture");
        self.camera.aspect = aspect_from_config(handle.config());
        self.shader.set_mvp(handle.queue(), self.camera.mvp());
    }
}

fn aspect_from_config(config: &wgpu::SurfaceConfiguration) -> f32 {
    config.width as f32 / config.height as f32
}
