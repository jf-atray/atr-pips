use crate::scripting::{Script, DomainView};

pub struct PhysicsSolver {
    pub gravity: f32,
}

impl PhysicsSolver {
    pub fn new() -> Self {
        Self { gravity: -9.8 }
    }
}

impl Script for PhysicsSolver {
    fn update(&mut self, _ctx: &DomainView) {
    }
}
