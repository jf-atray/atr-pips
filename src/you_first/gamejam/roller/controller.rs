use std::collections::HashMap;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::ecs::PipId;
use crate::gather::impls::gather_pair_mut;
use crate::input::Input;
use crate::you_first::gamejam::roller::components::RollerWorld;
use crate::you_first::gamejam::roller::projection::{WALK_SPEED, depth_factor_linear, world_x};

const LATERAL_SPEED: f32 = 2.1;

const PLAYER_X_MIN: f32 = -5.0;
const PLAYER_X_MAX: f32 = 5.0;

#[derive(Debug)]
pub struct PlayerLateralController {
    pub player: Option<PipId>,
}

impl Solver for PlayerLateralController {}

impl PlayerLateralController {
    pub fn new(player: Option<PipId>) -> Self {
        Self { player }
    }

    pub fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        input: &Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let (duel_active, walk_speed) = RollerWorld::signals(signals)
            .map(|s| (s.duel_active, s.walk_speed))
            .unwrap_or((false, WALK_SPEED));
        let lateral = if duel_active {
            0.0
        } else {
            input.axes.value("Horizontal")
        };

        let Some(player) = self.player else {
            return;
        };

        let Some(roller) = RollerWorld::tables(&mut pips.tables.pile) else {
            return;
        };
        let Some((player, depth)) = gather_pair_mut(
            &pips.ids,
            &mut roller.roller_players,
            &mut roller.roller_depths,
            player,
        ) else {
            return;
        };

        let t_ref = depth_factor_linear(0.0);
        let wx_ref = world_x(1.0, t_ref);
        let t_cur = depth_factor_linear(depth.d);
        let wx_cur = world_x(1.0, t_cur).max(0.01);
        let speed_scale = wx_ref / wx_cur;

        player.lateral = (player.lateral + lateral * LATERAL_SPEED * speed_scale * dt)
            .clamp(PLAYER_X_MIN, PLAYER_X_MAX);
        player.walk_distance += walk_speed * dt;
        depth.lateral = player.lateral;
    }
}
