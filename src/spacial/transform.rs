use glam::{*};

#[derive(Clone, Default, PartialEq)]
pub struct Transform {
    pub xyz: Vec3,
    pub rot: Quat,
}
impl Transform {
    pub fn with_xyz(&mut self, xyz: Vec3) -> &mut Self {
        self.xyz = xyz;
        self
    }
}