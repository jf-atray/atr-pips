use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Gravity {
    pub accel: Vec3,
}

impl Default for Gravity {
    fn default() -> Self {
        Self {
            accel: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}
