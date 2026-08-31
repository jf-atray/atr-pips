use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::input::Input;

#[derive(Debug)]
pub struct DiagnosticsSolver {
    last: Instant,
}

impl Solver for DiagnosticsSolver {}

impl DiagnosticsSolver {
    pub fn new() -> Self {
        Self { last: Instant::now() }
    }

    pub fn update(
        &mut self,
        _dt: f32,
        _pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        _input: &Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let now = Instant::now();
        let duration = now.duration_since(self.last);
        self.last = now;

        let Some(signals) = DiagnosticsAdd::signals(signals) else {
            return;
        };
        if signals.enabled {
            signals.duration = duration;
        }
    }
}

crate::addition! {
    #[derive(Debug)]
    pub struct diagnostics_world : DiagnosticsAdd {
        tables: {
            padding: Class<u8> = Class::new(GrowthStrategy::quart_kib::<u8>()),
        },
        solvers: {
            diagnostics: DiagnosticsSolver = DiagnosticsSolver::new(),
        },
        scripts: {},
        signals: {
            enabled: bool = false,
            duration: Duration = Duration::ZERO,
        },
    }
}
