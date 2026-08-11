use glam::Vec3;

use crate::query::impls::query_mut_mut;
use crate::scripting::{DomainView, Script};

const BOUNDS: f32 = 5.0;

pub struct PhysicsSolver {
    pub gravity: Vec3,
}

impl PhysicsSolver {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -2.0, 0.0),
        }
    }
}

impl Script for PhysicsSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt();
        let domain = ctx.domain();

        for (motions, xforms) in query_mut_mut(
            &mut domain.tables.core.motions,
            &(),
            &mut domain.tables.core.xforms,
            &(),
        ) {
            for (motion, xform) in motions.iter_mut().zip(xforms.iter_mut()) {
                motion.vel += self.gravity * dt;
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
