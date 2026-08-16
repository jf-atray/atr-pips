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

pub struct RollerProjectionSystem {
    player: PipId,
    ground_y: f32,
    horizon_y: f32,
}

impl RollerProjectionSystem {
    pub fn new(player: PipId) -> Self {
        Self {
            player,
            ground_y: 0.0,
            horizon_y: 0.0,
        }
    }
}

impl Script for RollerProjectionSystem {
    fn fixed_update(
        &mut self,
        world: &mut World,
        input: &InputContext,
        ctx: &mut SimulationContext,
    ) -> Option<SceneAction> {
        let state = ctx.shared.get_or_insert_default::<OverworldState>();
        let walk_speed = state.walk_speed;
        self.ground_y = state.ground_y;
        self.horizon_y = state.horizon_y;

        let mut to_despawn: Vec<PipId> = Vec::new();

        query!(
            |depth: &mut RollerDepth,
             pip: PipId| {
                if pip == self.player {
                    return;
                }

                depth.d -= (walk_speed + depth.speed) * input.dt;
                depth.lateral += depth.lateral_speed * input.dt;
                let t = depth_factor(depth.d);
                if t > DESPAWN_T {
                    to_despawn.push(pip);
                    return;
                }
                if depth.lateral.abs() > 20.0 {
                    to_despawn.push(pip);
                    return;
                }
            },
            &world.heading,
            [
                &mut world.tables.roller_depths
            ]
        );

        for pip in to_despawn {
            world.despawn(pip);
        }

        None
    }

    fn render_update(
        &mut self,
        world: &mut World,
        _input: &InputContext,
        _ctx: &mut PresentationContext,
    ) {
        let player_lateral = 0.0;
        let ground_y = self.ground_y;
        let horizon_y = self.horizon_y;
        let player_pip = self.player;

        query!(
            |transform: &mut Transform,
             brush: &mut Brush,
             sprite_rect: &mut SpriteRect,
             depth: &mut RollerDepth,
             pip: PipId| {
                let (pos, s) = project(depth.lateral, depth.d, player_lateral, depth.scalar, ground_y, horizon_y);
                transform.xyz = pos.extend(world_z(depth.d));
                brush.scale = Vec3::new(sprite_rect.w * s, sprite_rect.h * s, 1.0);
            },
            &world.heading,
            [
                &mut world.tables.transforms,
                &mut world.tables.brushes,
                &mut world.tables.sprite_rects,
                &mut world.tables.roller_depths
            ]
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
