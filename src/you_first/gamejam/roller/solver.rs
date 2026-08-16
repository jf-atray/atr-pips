use glam::Vec3;

use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::you_first::gamejam::roller::components::RollerAddition;
use crate::you_first::gamejam::roller::projection::{project, world_z, GROUND_Y, HORIZON_Y, WALK_SPEED};

#[derive(Debug, Default)]
pub struct RollerProjectionSolver;

impl Script for RollerProjectionSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let mut view = ctx.domain.tables.view();
        let Some(roller) = view.additions.get_mut::<RollerAddition>() else {
            return;
        };
        let core = &mut view.core;

        let dt = ctx.dt;

        let mut player_lateral = 0.0f32;
        crate::query!([& roller.roller_players], |player| {
            player_lateral = player.lateral;
        });

        crate::query!(
            [&mut roller.roller_depths, &mut core.xforms, &mut core.brushes],
            |depth, xform, brush| {
                depth.d -= (WALK_SPEED + depth.speed) * dt;
                depth.lateral += depth.lateral_speed * dt;

                let (pos, s) = project(
                    depth.lateral,
                    depth.d,
                    player_lateral,
                    depth.scalar,
                    GROUND_Y,
                    HORIZON_Y,
                );

                xform.xyz = Vec3::new(pos.x, pos.y, world_z(depth.d));

                let scale = s * depth.scalar;
                brush.scale = Vec3::new(scale, scale, 1.0);
            }
        );
    }
}
