pub struct Camera {
    pub(crate) eye: glam::Vec3,
    pub(crate) dir: glam::Vec3,
    pub(crate) up: glam::Vec3,
    pub(crate) aspect: f32,
    pub(crate) fov: f32,
    pub(crate) z_near: f32,
    pub(crate) z_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: (0.0, 1.0, 2.0).into(),
            dir: (0.0, 0.0, -1.0).into(),
            up: (0.0, 1.0, 0.0).into(),
            aspect: 1.0,
            fov: 60.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }
}

impl Camera {
    pub fn build_view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.eye, self.eye + self.dir, self.up)
    }

    pub fn build_proj_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_lh(self.fov.to_radians(), self.aspect, self.z_near, self.z_far)
    }

    pub fn build_view_proj_matrix(&self) -> glam::Mat4 {
        self.build_proj_matrix() * self.build_view_matrix()
    }
}
