pub trait Shader {
    fn render_pipeline(&self) -> &wgpu::RenderPipeline;
    fn bind_groups(&self, render_pass: &mut wgpu::RenderPass);
}

pub trait UseShader {
    fn set_shader<S: Shader>(&mut self, shader: &S);
}

impl<'a> UseShader for wgpu::RenderPass<'a> {
    fn set_shader<S: Shader>(&mut self, shader: &S) {
        self.set_pipeline(shader.render_pipeline());
        shader.bind_groups(self);
    }
}
