use crate::query::impls::query_mut_mut;
use crate::tables::tables::Tables;

const BOUNDS: f32 = 5.0;

pub struct MotionSolver;

impl MotionSolver {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, tables: &mut Tables, dt: f32) {
        for (motions, xforms) in query_mut_mut(&mut tables.core.motions, &(), &mut tables.core.xforms, &()) {
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
