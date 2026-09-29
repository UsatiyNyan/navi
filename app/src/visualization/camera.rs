pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn mvp(&self) -> glam::Mat4 {
        #[cfg(not(target_arch = "wasm32"))]
        use glam::camera::rh::proj::opengl::perspective;

        // directx and webgpu use the same perspective matrix
        #[cfg(target_arch = "wasm32")]
        use glam::camera::rh::proj::directx::perspective;

        let view = glam::camera::rh::view::look_at_mat4(self.eye, self.target, self.up);
        let proj = perspective(self.fovy, self.aspect, self.znear, self.zfar);
        return proj * view;
    }
}
