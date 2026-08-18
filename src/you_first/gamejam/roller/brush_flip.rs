use crate::brushes::Brush;
use crate::query;
use crate::query::impls::query_mut_mut;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::you_first::gamejam::roller::components::{BrushFlip, RollerAddition, solve_flip};

pub struct BrushFlipSolver;

impl Script for BrushFlipSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let mut tables = ctx.domain.tables.view();
        let Some(roller) = tables.additions.get_mut::<RollerAddition>() else {
            return;
        };
        query!(
            [&mut tables.core.brushes, &mut roller.brush_flips],
            |brush, flip| {
                solve_flip(brush, flip);
            }
        );
    }
}
