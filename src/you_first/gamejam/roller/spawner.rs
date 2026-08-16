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

pub struct RollerSpawner {
    player: PipId,
    biome: Biome,
    rng_state: u32,
    initialized: bool,
    biome_swapped: bool,
}

impl RollerSpawner {
    pub fn new(player: PipId) -> Self {
        Self {
            player,
            biome: Biome::sparse_desert(),
            rng_state: 12345,
            initialized: false,
            biome_swapped: false,
        }
    }
}

impl Script for RollerSpawner {
    fn fixed_update(
        &mut self,
        world: &mut World,
        _input: &InputContext,
        ctx: &mut SimulationContext,
    ) -> Option<SceneAction> {
        use crate::gather;
        let Some(player_state) = gather!(
            self.player,
            &world.heading,
            [&world.tables.roller_players]
        ) else {
            return None;
        };
        let walk_distance = player_state.walk_distance;

        if !self.initialized {
            let living_anim = *ctx.shared.get_or_insert_default::<crate::gamejam::duel::state::LivingAnimLib>();
            let tumbleweed_anim = *ctx.shared.get_or_insert_default::<crate::gamejam::duel::state::TumbleweedAnimLib>();
            self.biome.pre_seed(world, ctx.resources, living_anim, tumbleweed_anim, &mut self.rng_state);
            self.initialized = true;
        }

        //swap from sparse desert to default desert after walking 0.1 units.
        if !self.biome_swapped && walk_distance >= 0.1 {
            self.biome = Biome::default_desert();
            self.biome_swapped = true;
            log::info!("biome: swapped to default desert at walk_distance={}", walk_distance);
        }

        let living_anim = *ctx.shared.get_or_insert_default::<crate::gamejam::duel::state::LivingAnimLib>();
        let tumbleweed_anim = *ctx.shared.get_or_insert_default::<crate::gamejam::duel::state::TumbleweedAnimLib>();
        self.biome.update(world, ctx.resources, living_anim, tumbleweed_anim, walk_distance, &mut self.rng_state);

        None
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
