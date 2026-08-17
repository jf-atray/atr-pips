use crate::gather::impls::gather_pair_mut;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::tables::PipId;
use crate::you_first::gamejam::roller::components::RollerAddition;
use crate::you_first::gamejam::roller::projection::{WALK_SPEED, depth_factor_linear, world_x};

const LATERAL_SPEED: f32 = 2.1;

const PLAYER_X_MIN: f32 = -5.0;
const PLAYER_X_MAX: f32 = 5.0;

pub struct PlayerLateralController {
    player: PipId,
}

impl PlayerLateralController {
    pub fn new(player: PipId) -> Self {
        Self { player }
    }
}

impl Script for PlayerLateralController {
    fn update(&mut self, ctx: &mut DomainView) {
        let lateral = ctx.input.axes.value("Horizontal");

        let Some(roller) = ctx.domain.tables.get_mut::<RollerAddition>() else {
            return;
        };
        let Some((player, depth)) = gather_pair_mut(
            &ctx.domain.ids,
            &mut roller.roller_players,
            &mut roller.roller_depths,
            self.player,
        ) else {
            return;
        };

        let t_ref = depth_factor_linear(0.0);
        let wx_ref = world_x(1.0, t_ref);
        let t_cur = depth_factor_linear(depth.d);
        let wx_cur = world_x(1.0, t_cur).max(0.01);
        let speed_scale = wx_ref / wx_cur;

        player.lateral =
            (player.lateral + lateral * LATERAL_SPEED * speed_scale * ctx.dt)
                .clamp(PLAYER_X_MIN, PLAYER_X_MAX);
        player.walk_distance += WALK_SPEED * ctx.dt;
        depth.lateral = player.lateral;
    }
}
