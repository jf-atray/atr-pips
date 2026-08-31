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

    #[allow(clippy::unused_self, reason = "called via solver dispatch")]
    pub fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        _input: &mut Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let core = &mut pips.tables.core;
        let boundary = &signals.core.boundary;

        crate::query!(
            [
                &mut core.motions,
                &mut core.xforms
            ],
            |motion, xform| {
                let mut next = xform.xyz + motion.vel * dt;
                boundary.reflect(&mut next, &mut motion.vel);
                xform.xyz = next;
            }
        );
    }
}
