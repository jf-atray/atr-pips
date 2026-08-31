use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver, Tables};
use crate::assets::SpriteEntry;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::input::{AxisConfig, Input};
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct TableReport {
    pub class_count: usize,
    pub row_count: usize,
    pub capacity: usize,
    pub byte_size: usize,
}

#[derive(Debug)]
pub struct DiagnosticsSolver {
    last: Instant,
    last_print_key: Option<KeyCode>,
}

impl Solver for DiagnosticsSolver {}

impl DiagnosticsSolver {
    pub fn new() -> Self {
        Self {
            last: Instant::now(),
            last_print_key: None,
        }
    }

    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        input: &mut Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let now = Instant::now();
        let duration = now.duration_since(self.last);
        self.last = now;

        let Some(signals) = DiagnosticsAdd::signals(signals) else {
            return;
        };

        if self.last_print_key != signals.print_key {
            input.add_axis(
                "diagnostics_print",
                AxisConfig::new(signals.print_key.into_iter().collect(), vec![]),
            );
            self.last_print_key = signals.print_key;
        }

        if !signals.enabled {
            return;
        }

        signals.duration = duration;
        signals.table_inventory.clear();

        let mut collect = |name: &'static str, table: &dyn crate::ecs::Table| {
            signals.table_inventory.push((
                name,
                TableReport {
                    class_count: table.class_count(),
                    row_count: table.row_count(),
                    capacity: table.capacity(),
                    byte_size: table.byte_size(),
                },
            ));
        };

        pips.tables.core.for_each_table(&mut collect);
        for tables in pips.tables.iter_mut() {
            tables.for_each_table(&mut collect);
        }

        if input.axes.delta("diagnostics_print") == Some(1) {
            log::info!("{signals:#?}");
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
            table_inventory: Vec<(&'static str, TableReport)> = Vec::new(),
            print_key: Option<KeyCode> = Some(KeyCode::KeyP),
        },
    }
}
