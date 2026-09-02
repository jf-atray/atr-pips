use std::collections::HashMap;

use glam::Quat;

use crate::addition::{Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::ecs::scope::Scope;
use crate::ecs::PipId;
use crate::input::Input;
use crate::spacial::motion::MotionKind;

const SLEEP_THRESHOLD: f32 = 0.4;

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
        let drag = signals.core.drag;

        crate::query!(
            [
                MotionKind::Active; &mut core.motions,
                (); &mut core.xforms
            ],
            |motion, xform| {
                if drag > 0.0 {
                    let decay = (1.0 - drag).powf(dt);
                    motion.vel *= decay;
                    motion.ang_vel *= decay;
                }
                let mut next = xform.xyz + motion.vel * dt;
                boundary.reflect(&mut next, &mut motion.vel);
                xform.xyz = next;
                xform.rot = Quat::from_rotation_z(motion.ang_vel * dt) * xform.rot;
            }
        );

        self.sleep_slow_pips(pips);
    }

    fn sleep_slow_pips(&mut self, pips: &mut Pips) {
        let mut to_sleep: Vec<PipId> = Vec::new();
        {
            let core = &pips.tables.core;
            let pip_ids = &pips.pip_ids;
            for (class_id, col) in core.motions.data.iter() {
                if col.key != MotionKind::Active {
                    continue;
                }
                let Some(pip_col) = pip_ids.data.get(class_id) else {
                    continue;
                };
                for (row_idx, motion) in col.iter().enumerate() {
                    if motion.vel.length() < SLEEP_THRESHOLD && motion.ang_vel.abs() < SLEEP_THRESHOLD {
                        if let Some(&pip) = pip_col.get(row_idx) {
                            to_sleep.push(pip);
                        }
                    }
                }
            }
        }

        for pip in to_sleep {
            pips.move_pip(pip, |scope: &mut Scope| {
                if let Some((_m, k)) = &mut scope.core.motions {
                    *k = MotionKind::Sleeping;
                }
            });
        }
    }
}
