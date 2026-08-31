use std::collections::HashMap;

use glam::Vec3;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::input::Input;
use crate::physics::PhysicsAdd;
use crate::query;

#[derive(Debug)]
pub struct PhysicsSolver;

impl Solver for PhysicsSolver {}

impl PhysicsSolver {
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
        let Some(physics) = PhysicsAdd::tables(&mut pips.tables.pile) else {
            return;
        };
        let Some(signals) = PhysicsAdd::signals(signals) else {
            return;
        };
        let gravity = signals.gravity.accel * dt;

        query!([
            &mut physics.inv_masses,
            &mut physics.impulses,
            &mut core.motions,
        ], |inv_mass, impulse, motion| {
            if inv_mass.is_normal() {
                let massed_impulse = *impulse * *inv_mass;
                motion.vel += massed_impulse + gravity;
                *impulse = Vec3::ZERO;
            }
        });
    }
}
