use glam::Vec2;

use crate::addition;
use crate::brushes::Brush;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;

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

addition! {
    #[derive(Debug)]
    pub struct roller_world : RollerWorld {
        tables: {
            roller_depths: Class<RollerDepth> = Class::new(GrowthStrategy::quart_kib::<RollerDepth>()),
            roller_players: Class<RollerPlayer> = Class::new(GrowthStrategy::quart_kib::<RollerPlayer>()),
            brush_flips: Class<BrushFlip> = Class::new(GrowthStrategy::quart_kib::<BrushFlip>()),
        },
        solvers: {
            brush_flip: super::brush_flip::BrushFlipSolver = super::brush_flip::BrushFlipSolver,
            cloud_drift: super::clouds::CloudDriftSystem = super::clouds::CloudDriftSystem::new(),
        },
        scripts: {},
        signals: {},
    }
}

#[allow(unused_imports)]
pub use roller_world::Tables as RollerAddition;
pub use roller_world::View as RollerView;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct BrushFlip {
    pub is_flipped: bool,
}

pub fn solve_flip(brush: &mut Brush, flip: &BrushFlip) {
    let sign = if flip.is_flipped { -1.0 } else { 1.0 };
    brush.scale.x = brush.scale.x.copysign(sign);
}
