use crate::addition::{Pips, Solver};
use crate::query;
use crate::you_first::gamejam::roller::components::{RollerWorld, solve_flip};

#[derive(Debug)]
pub struct BrushFlipSolver;

impl Solver for BrushFlipSolver {}

impl BrushFlipSolver {
    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut crate::addition::ScriptsMap,
        _signals: &mut crate::addition::SignalsMap,
        _input: &crate::input::Input,
        _asset_registry: &std::collections::HashMap<String, crate::assets::SpriteEntry>,
    ) {
        let Some(roller) = RollerWorld::tables(&mut pips.tables) else {
            return;
        };
        query!(
            [&mut pips.tables.core.brushes, &mut roller.brush_flips],
            |brush, flip| {
                solve_flip(brush, flip);
            }
        );
    }
}
