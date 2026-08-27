use crate::addition::{Addition, Solver};
use crate::query;
use crate::you_first::gamejam::roller::components::{RollerWorld, solve_flip};

#[derive(Debug)]
pub struct BrushFlipSolver;

impl Solver for BrushFlipSolver {}

impl BrushFlipSolver {
    pub fn update(
        &mut self,
        _dt: f32,
        tables: &mut crate::addition::TablesMap,
        _scripts: &mut crate::addition::SolversMap,
        _signals: &mut crate::addition::SignalsMap,
    ) {
        let Some(roller) = RollerWorld::tables(tables) else {
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
