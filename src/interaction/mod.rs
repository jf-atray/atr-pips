use std::collections::HashMap;

use glam::Vec2;

use crate::addition;
use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::assets::SpriteEntry;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::ecs::PipId;
use crate::input::Input;

#[derive(Clone, Copy, Debug)]
pub struct Clickable {
    pub half_extent: Vec2,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PickResult {
    pub clicked: Option<PipId>,
    pub hovered: Option<PipId>,
}

#[derive(Debug)]
pub struct PickSolver;

impl Solver for PickSolver {}

impl PickSolver {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::unused_self, reason = "called via solver dispatch")]
    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        input: &mut Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let mouse_world = signals.core.mouse_world;
        let clicked_this_frame = input.axes.delta("click") == Some(1);

        let Some(interaction) = InteractionAdd::tables(&mut pips.tables.pile) else { return };
        let Some(interaction_signals) = InteractionAdd::signals(signals) else { return };

        let mut hovered: Option<PipId> = None;
        let clickables = &interaction.clickables;
        let xforms = &pips.tables.core.xforms;

        for (class_id, click_col) in clickables.data.iter() {
            let Some(xform_col) = xforms.data.get(class_id) else { continue };
            let Some(pip_col) = pips.pip_ids.data.get(class_id) else { continue };

            for i in 0..click_col.len() {
                let clickable = &click_col[i];
                let xform = &xform_col[i];
                let pos = xform.xyz;

                if mouse_world.x >= pos.x - clickable.half_extent.x
                    && mouse_world.x <= pos.x + clickable.half_extent.x
                    && mouse_world.y >= pos.y - clickable.half_extent.y
                    && mouse_world.y <= pos.y + clickable.half_extent.y
                {
                    hovered = Some(pip_col[i]);
                    break;
                }
            }
            if hovered.is_some() {
                break;
            }
        }

        let clicked = if clicked_this_frame { hovered } else { None };

        interaction_signals.pick_result = PickResult { clicked, hovered };
    }
}

addition! {
    #[derive(Debug)]
    pub struct interaction_world : InteractionAdd {
        tables: {
            clickables: Class<Clickable> = Class::new(GrowthStrategy::quart_kib::<Clickable>()),
        },
        solvers: {
            pick: PickSolver = PickSolver::new(),
        },
        scripts: {},
        signals: {
            pick_result: PickResult = PickResult::default(),
        },
    }
}
