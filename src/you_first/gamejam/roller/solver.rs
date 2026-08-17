use glam::{Vec2, Vec3};

use crate::brushes::Brush;
use crate::query;
use crate::query::impls::query_mut_mut_mut;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::spacial::transform::Transform;
use crate::tables::PipId;
use crate::you_first::gamejam::roller::components::{RollerAddition, RollerDepth};
use crate::you_first::gamejam::roller::projection::{
    DESPAWN_T, GROUND_Y, HORIZON_Y, WALK_SPEED, project, world_z,
};

pub struct RollerProjectionSolver;

impl Script for RollerProjectionSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let mut to_despawn: Vec<PipId> = Vec::new();

        {
            let tables = &mut ctx.domain.tables;
            let Some(roller) = tables.get::<RollerAddition>() else {
                return;
            };


            query!([&roller.roller_depths, &ctx.domain.tables.system.pip_id], |rd, id| {
                rd.d -= (WALK_SPEED + rd.speed) * ctx.dt;
                rd.lateral += rd.lateral_speed * ctx.dt;

                if rd.d < DESPAWN_T || rd.lateral.abs() > 20.0 {
                    to_despawn.push(id.clone());
                }
            });
            query!([&mut roller.roller_depths, &mut ctx.domain.tables.core.xforms, &mut ctx.domain.tables.core.brushes], |rd, xform, brush| {
                let (pos, s) = project(rd.lateral, rd.d, 0.0, rd.scalar, GROUND_Y, HORIZON_Y);
                xform.xyz = pos.extend(world_z(rd.d));
                brush.scale = Vec3::new(s * rd.scalar, s * rd.scalar, 1.0);
            });
        }

        for pip in to_despawn {
            ctx.domain.destroy(pip);
        }
    }
}
