use crate::scripting::context::DomainView;
use crate::scripting::script::Script;

#[derive(Debug)]
pub struct MotionSolver;

impl MotionSolver {
    pub fn new() -> Self {
        Self
    }
}

impl Script for MotionSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt;
        let core = &mut ctx.domain.pips.tables.core;
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
