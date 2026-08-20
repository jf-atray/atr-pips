use glam::{Quat, Vec3};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Transform {
    pub xyz: Vec3,
    pub rot: Quat,
}
impl Transform {
    pub fn xyz(&mut self, xyz: Vec3) -> &mut Self {
        self.xyz = xyz;
        self
    }
}
