use glam::Mat4;

#[derive(Clone, Copy, Debug, Default)]
pub struct Camera {
    pub view_proj: Mat4,
}
