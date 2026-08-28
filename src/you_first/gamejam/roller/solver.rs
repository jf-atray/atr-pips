use std::collections::HashMap;

use glam::Vec3;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::ecs::PipId;
use crate::input::Input;
use crate::query;
use crate::you_first::gamejam::roller::components::RollerWorld;
use crate::you_first::gamejam::roller::projection::{
    DESPAWN_T, GROUND_Y, HORIZON_Y, WALK_SPEED, depth_factor, project, world_z,
};

#[derive(Debug)]
pub struct RollerProjectionSolver {
    pub player: Option<PipId>,
}

impl Solver for RollerProjectionSolver {}

impl RollerProjectionSolver {
    pub fn new(player: Option<PipId>) -> Self {
        Self { player }
    }

    pub fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        _input: &Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let Some(roller) = RollerWorld::tables(&mut pips.tables.pile) else {
            return;
        };

        let player = self.player;
        let walk_speed = RollerWorld::signals(signals)
            .map(|s| s.walk_speed)
            .unwrap_or(WALK_SPEED);
        let mut to_despawn: Vec<PipId> = Vec::new();

        query!(
            [(); &mut roller.roller_depths, (); &pips.pip_ids],
            |depth, pip_id| {
                if Some(*pip_id) == player {
                    return;
                }

                depth.d -= (walk_speed + depth.speed) * dt;
                depth.lateral += depth.lateral_speed * dt;

                if depth_factor(depth.d) > DESPAWN_T {
                    to_despawn.push(*pip_id);
                } else if depth.lateral.abs() > 20.0 {
                    to_despawn.push(*pip_id);
                }
            }
        );

        for pip in to_despawn {
            pips.destroy(pip);
        }

        let core = &mut pips.tables.core;
        let Some(roller) = RollerWorld::tables(&mut pips.tables.pile) else {
            return;
        };

        query!(
            [
                &mut roller.roller_depths,
                &mut core.xforms,
                &mut core.brushes
            ],
            |rd, xform, brush| {
                let (pos, s) = project(rd.lateral, rd.d, 0.0, rd.scalar, GROUND_Y, HORIZON_Y);
                xform.xyz = pos.extend(world_z(rd.d));
                brush.scale = Vec3::new(
                    rd.base_scale.x * s,
                    rd.base_scale.y * s,
                    1.0,
                );
            }
        );
    }
}
