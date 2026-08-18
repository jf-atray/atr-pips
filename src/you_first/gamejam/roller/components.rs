use glam::Vec2;

use crate::brushes::Brush;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RollerDepth {
    pub d: f32,
    pub lateral: f32,
    pub speed: f32,
    pub scalar: f32,
    pub lateral_speed: f32,
    /// World-space base scale before depth projection and `scalar` are applied.
    /// Compute as `desired_world_size / sprite.natural_scale` at spawn.
    pub base_scale: Vec2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RollerPlayer {
    pub walk_distance: f32,
    pub lateral: f32,
}

crate::partition! {
    pub struct RollerAddition as RollerView {
        pub roller_depths: Class<RollerDepth>,
        pub roller_players: Class<RollerPlayer>,
        pub brush_flips: Class<BrushFlip>,
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct BrushFlip {
    pub is_flipped: bool,
}

pub fn solve_flip(brush: &mut Brush, flip: &BrushFlip) {
    let sign = if flip.is_flipped { -1.0 } else { 1.0 };
    brush.scale.x = brush.scale.x.copysign(sign);
}
