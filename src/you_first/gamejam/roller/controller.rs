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

const LATERAL_SPEED: f32 = 2.1;
const DEPTH_SPEED: f32 = 1.7;

const PLAYER_X_MIN: f32 = -5.0;
const PLAYER_X_MAX: f32 = 5.0;
const PLAYER_Y_MIN: f32 = 1.0;
const PLAYER_Y_MAX: f32 = 4.8;

pub struct PlayerLateralController {
    player: PipId,
}

impl PlayerLateralController {
    pub fn new(player: PipId) -> Self {
        Self { player }
    }
}

impl Script for PlayerLateralController {
    fn fixed_update(
        &mut self,
        world: &mut World,
        input: &InputContext,
        ctx: &mut SimulationContext,
    ) -> Option<SceneAction> {
        use crate::gather;

        if ctx.shared.get_or_insert_default::<PauseState>().paused {
            return None;
        }

        let Some((player_state, depth)) = gather!(
            self.player,
            &world.heading,
            [
                &mut world.tables.roller_players,
                &mut world.tables.roller_depths
            ]
        ) else {
            return None;
        };

        let walk_speed = ctx
            .shared
            .get::<OverworldState>()
            .map(|s| s.walk_speed)
            .unwrap_or(1.0);
        let dodge_mul = ctx
            .shared
            .get::<OverworldState>()
            .map(|s| s.dodge_speed_mul)
            .unwrap_or(1.0);

        let mut lateral_delta = 0.0f32;
        let mut depth_delta = 0.0f32;
        if input.snapshot.held.arrow_left || input.snapshot.held.a {
            lateral_delta -= 1.0;
        }
        if input.snapshot.held.arrow_right || input.snapshot.held.d {
            lateral_delta += 1.0;
        }
        if input.snapshot.held.w || input.snapshot.held.arrow_up {
            depth_delta += 1.0;
        }
        if input.snapshot.held.s || input.snapshot.held.arrow_down {
            depth_delta -= 1.0;
        }

        let t_ref = depth_factor_linear(0.0);
        let wx_ref = world_x(1.0, t_ref);
        let t_cur = depth_factor_linear(depth.d);
        let wx_cur = world_x(1.0, t_cur).max(0.01);
        let speed_scale = wx_ref / wx_cur;

        player_state.lateral = (player_state.lateral + lateral_delta * LATERAL_SPEED * speed_scale * dodge_mul * input.dt)
            .clamp(PLAYER_X_MIN, PLAYER_X_MAX);
        player_state.walk_distance += walk_speed * input.dt;

        // During a duel the autowalk is paused, so W/S let the player
        // move forward/back in the roller depth while S does the reverse.
        // But during ScurryLoss the DuelConductor owns depth â€” don't clamp.
        let scurrying = ctx
            .shared
            .get::<OverworldState>()
            .is_some_and(|s| s.duel_scurrying);
        if walk_speed == 0.0 && !scurrying {
            depth.d += depth_delta * DEPTH_SPEED * input.dt;
            depth.d = depth.d.clamp(PLAYER_Y_MIN, PLAYER_Y_MAX);
        }

        depth.lateral = player_state.lateral;

        None
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn player_lateral(world: &World, player: PipId) -> f32 {
    use crate::gather;
    gather!(player, &world.heading, [&world.tables.roller_players])
        .map(|p| p.lateral)
        .unwrap_or(0.0)
}
