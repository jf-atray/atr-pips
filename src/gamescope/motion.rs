use std::collections::HashMap;

use crate::addition::{Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::input::Input;

#[derive(Debug)]
pub struct MotionSolver;

impl Solver for MotionSolver {}

impl MotionSolver {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        _signals: &mut SignalsMap,
        _input: &Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let core = &mut pips.tables.core;
        crate::query!(
            [
                &mut core.motions,
                &mut core.xforms
            ],
            |motion, xform| {
                xform.xyz += motion.vel * dt;
            }
        );
    }
}
