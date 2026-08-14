use crate::query::impls::query_mut_mut;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;

const BOUNDS: f32 = 5.0;

pub struct MotionSolver;

impl MotionSolver {
    pub fn new() -> Self {
        Self
    }

}

impl Script for MotionSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt;
        for (motions, xforms) in query_mut_mut(&mut ctx.domain.tables.core.motions, &(), &mut ctx.domain.tables.core.xforms, &()) {
            for (motion, xform) in motions.iter_mut().zip(xforms.iter_mut()) {
                xform.xyz += motion.vel * dt;
                for axis in 0..3 {
                    if xform.xyz[axis] > BOUNDS {
                        xform.xyz[axis] = BOUNDS;
                        motion.vel[axis] = -motion.vel[axis];
                    } else if xform.xyz[axis] < -BOUNDS {
                        xform.xyz[axis] = -BOUNDS;
                        motion.vel[axis] = -motion.vel[axis];
                    }
                }
            }
        }
    }
}
