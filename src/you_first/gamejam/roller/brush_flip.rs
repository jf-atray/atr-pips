use crate::addition::pair_tables;
use crate::query;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::ecs::core::CoreWorld;
use crate::you_first::gamejam::roller::components::{RollerWorld, solve_flip};

#[derive(Debug)]
pub struct BrushFlipSolver;

impl Script for BrushFlipSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let Some((core, roller)) = pair_tables::<CoreWorld, RollerWorld>(&mut ctx.domain.tables) else {
            return;
        };
        query!(
            [&mut core.brushes, &mut roller.brush_flips],
            |brush, flip| {
                solve_flip(brush, flip);
            }
        );
    }
}
