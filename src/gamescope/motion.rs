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
        crate::query!(
            [&mut ctx.domain.tables.core.motions, &mut ctx.domain.tables.core.xforms],
            |motion, xform| {
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
        );
    }
}
