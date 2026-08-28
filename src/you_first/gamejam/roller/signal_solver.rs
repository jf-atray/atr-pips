use std::collections::HashMap;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::input::Input;
use crate::you_first::gamejam::duel::components::DuelWorld;
use crate::you_first::gamejam::duel::state::DuelState;
use crate::you_first::gamejam::roller::components::RollerWorld;
use crate::you_first::gamejam::roller::projection::WALK_SPEED;

#[derive(Debug)]
pub struct RollerStateSolver;

impl Solver for RollerStateSolver {}

impl RollerStateSolver {
    pub fn update(
        &mut self,
        _dt: f32,
        _pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        _input: &Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let duel_active = DuelWorld::signals(signals)
            .map(|s| !matches!(&s.duel_state, DuelState::Inactive))
            .unwrap_or(false);

        if let Some(roller) = RollerWorld::signals(signals) {
            roller.walk_speed = if duel_active { 0.0 } else { WALK_SPEED };
        }
    }
}
