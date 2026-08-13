use glam::{Mat4, Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pos: Vec2,
    pub zoom: f32,
    pub rot: f32,
    pub view_proj: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: Vec2::ZERO,
            zoom: 1.0,
            rot: 0.0,
            view_proj: Mat4::IDENTITY,
        }
    }

    pub fn update(&mut self, aspect: f32) {
        //the bigger dim is 10 and the smaller scales w/ aspect.
        let (w, h) = if aspect >= 1.0 {
            (10.0, 10.0 / aspect)
        } else {
            (10.0 * aspect, 10.0)
        };
        let half_w = (w * 0.5) / self.zoom;
        let half_h = (h * 0.5) / self.zoom;

        //todo, make sure im rows columning correct
        let projection = glam::camera::lh::proj::directx::orthographic(-half_w, half_w, -half_h, half_h, -1.0, 1.0);
        let view = Mat4::from_translation(Vec3::new(-self.pos.x, -self.pos.y, 0.0))
            * Mat4::from_rotation_z(-self.rot);
        self.view_proj = projection * view;
    }
}

#[derive(Clone, Debug)]
pub struct CameraController {
    pub pan_speed: f32,
}

impl CameraController {
    pub fn new(pan_speed: f32) -> Self {
        Self { pan_speed }
    }
}
