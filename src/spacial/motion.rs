use glam::Vec3;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionKind {
    Static,
    Active,
    Sleeping,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Motion {
    pub vel: Vec3,
    pub ang_vel: f32,
}

impl Motion {
    pub fn random_unit() -> Self {
        let mut rng = rand::rng();
        Self {
            vel: Vec3::new(
                rng.random::<f32>() * 2.0 - 1.0,
                rng.random::<f32>() * 2.0 - 1.0,
                0.0,
            )
            .normalize_or_zero(),
            ang_vel: 0.0,
        }
    }
}
