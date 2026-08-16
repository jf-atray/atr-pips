use glam::{Vec2, Vec3};

use crate::gamejam::roller::biome::Biome;
use crate::gamejam::roller::components::{RollerDepth, RollerPlayer};
use crate::gamejam::bundles::SpriteBundle;
use crate::query;
use crate::gamejam::roller::projection::{
    DESPAWN_T, FAR_Z, depth_factor, depth_factor_linear,
    project, world_x, world_z,
};
use crate::pip::{Transform, brush::Brush, sprite_rect::SpriteRect};
use crate::scenes::SceneAction;
use crate::scripts::{InputContext, PresentationContext, Script, SimulationContext};
use crate::tables::PipId;
use crate::world::World;

const CLOUD_SPRITES: &[(&str, f32)] = &[
    ("cloud_1", 4.0),  // 2x0.5
    ("cloud_2", 1.0),  // 1x1
];

const CLOUD_DRIFT_SPEED: f32 = 0.075;

const CLOUD_WRAP_X: f32 = 6.0;

const CLOUD_COUNT: usize = 4;

const CLOUD_Z: f32 = 0.9;

pub struct CloudDriftSystem {
    clouds: Vec<PipId>,
    initialized: bool,
    rng_state: u32,
}

impl CloudDriftSystem {
    pub fn new() -> Self {
        Self {
            clouds: Vec::new(),
            initialized: false,
            rng_state: 98765,
        }
    }

    fn next_random(&mut self) -> f32 { //todo don't we have rand crate?
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_state & 0x7fff) as f32) / 32768.0
    }

    fn spawn_clouds(&mut self, world: &mut World, resources: &crate::scenes::SceneResources<'_>) {
        for _ in 0..CLOUD_COUNT {
            let x = (self.next_random() * 2.0 - 1.0) * CLOUD_WRAP_X;
            let y = 0.5 + self.next_random() * 1.0;
            let w = 1.5 + self.next_random() * 1.5;
            let (sprite, aspect) = CLOUD_SPRITES[(self.next_random() * CLOUD_SPRITES.len() as f32) as usize % CLOUD_SPRITES.len()];
            let h = w / aspect;

            let pip = SpriteBundle::from_asset(
                Vec3::new(x, y, CLOUD_Z),
                Vec2::new(w, h),
                [1.0, 1.0, 1.0, 1.0],
                sprite,
                resources,
            )
            .spawn(world);
            self.clouds.push(pip);
        }
    }
}

impl Script for CloudDriftSystem {
    fn fixed_update(
        &mut self,
        world: &mut World,
        input: &InputContext,
        ctx: &mut SimulationContext,
    ) -> Option<SceneAction> {
        if !self.initialized {
            self.spawn_clouds(world, ctx.resources);
            self.initialized = true;
        }

        for &pip in &self.clouds {
            use crate::gather;
            if let Some(transform) = gather!(
                pip,
                &world.heading,
                [&mut world.tables.transforms]
            ) {
                transform.xyz.x += CLOUD_DRIFT_SPEED * input.dt;
                if transform.xyz.x > CLOUD_WRAP_X {
                    transform.xyz.x = -CLOUD_WRAP_X;
                }
            }
        }

        None
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
