use glam::Vec3;

use crate::addition::{Addition, pair_tables};
use crate::query;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::ecs::PipId;
use crate::ecs::core::CoreWorld;
use crate::ecs::system::SystemWorld;
use crate::you_first::gamejam::roller::components::RollerWorld;
use crate::you_first::gamejam::roller::projection::{
    DESPAWN_T, GROUND_Y, HORIZON_Y, WALK_SPEED, depth_factor, project, world_z,
};

#[derive(Debug)]
pub struct RollerProjectionSolver;

impl Script for RollerProjectionSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let mut to_despawn: Vec<PipId> = Vec::new();

        {
            let Some((roller, system)) = pair_tables::<RollerWorld, SystemWorld>(&mut ctx.domain.pips.tables) else {
                return;
            };

            query!(
                [&mut roller.roller_depths, &mut system.pip_id],
                |rd, id| {
                    rd.d -= (WALK_SPEED + rd.speed) * ctx.dt;
                    rd.lateral += rd.lateral_speed * ctx.dt;

                    if depth_factor(rd.d) > DESPAWN_T || rd.lateral.abs() > 20.0 {
                        to_despawn.push(*id);
                    }
                }
            );
        }

        {
            let Some(roller) = RollerWorld::tables(&mut ctx.domain.pips.tables) else {
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

        for pip in to_despawn {
            ctx.domain.destroy(pip);
        }
    }
}
