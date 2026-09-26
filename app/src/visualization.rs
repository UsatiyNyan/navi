use super::app;
use lib::{buffer, render};
use wgpu;

pub struct Visualization {}

impl Visualization {
    pub fn new(render_handle: &render::Handle) -> Self {
        Self {}
    }

    pub fn render(
        &self,
        model: &app::Model,
        render_handle: &mut render::Handle,
        buffer_state: &mut buffer::State<app::Record>,
    ) -> anyhow::Result<()> {
        // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("Render Pass"),
        //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //         view: &view,
        //         resolve_target: None,
        //         depth_slice: None,
        //         ops: wgpu::Operations {
        //             load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
        //             store: wgpu::StoreOp::Store,
        //         },
        //     })],
        //     depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        //         view: &self.depth_texture.view,
        //         depth_ops: Some(wgpu::Operations {
        //             load: wgpu::LoadOp::Clear(1.0),
        //             store: wgpu::StoreOp::Store,
        //         }),
        //         stencil_ops: None,
        //     }),
        //     occlusion_query_set: None,
        //     timestamp_writes: None,
        //     multiview_mask: None,
        // });
        Ok(())
    }
}
