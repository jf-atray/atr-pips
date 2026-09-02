use std::collections::HashMap;

use crate::addition::{Addition, Pips, Polypile, ScriptsMap, SignalsMap, Solver, Tables as AdditionTables};
use crate::assets::SpriteEntry;
use crate::broadphase::BroadPhaseAdd;
use crate::ecs::core::CoreAdd;
use crate::ecs::gather::impls::gather_ref;
use crate::input::Input;
use crate::spacial::aabb::Aabb;
use crate::spacial::motion::MotionKind;

#[derive(Debug)]
pub struct BroadPhaseSolver;

impl Solver for BroadPhaseSolver {}

impl BroadPhaseSolver {
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
        _input: &mut Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let Some(broad_signals) = BroadPhaseAdd::signals(signals) else {
            return;
        };

        Self::update_aabbs(&mut pips.tables.core, &mut pips.tables.pile);
        Self::rebuild_hash(&pips.pip_ids, &mut pips.tables.core, &mut pips.tables.pile, &mut broad_signals.hash);
        Self::generate_pairs(&pips.ids, &pips.tables.pile, &broad_signals.hash, &mut broad_signals.pairs);
    }

    fn update_aabbs(
        core: &mut <CoreAdd as Addition>::Tables,
        pile: &mut Polypile<dyn AdditionTables>,
    ) {
        let Some(broad) = BroadPhaseAdd::tables(pile) else { return };

        for (class_id, motion_col) in core.motions.data.iter() {
            if motion_col.key != MotionKind::Active {
                continue;
            }
            let Some(xform_col) = core.xforms.data.get_mut(class_id) else {
                continue;
            };
            let Some(brush_col) = core.brushes.data.get(class_id) else {
                continue;
            };
            let Some(aabb_col) = broad.aabbs.data.get_mut(class_id) else {
                continue;
            };
            for i in 0..xform_col.len() {
                let extent = brush_col[i].scale * 0.5;
                aabb_col[i] = Aabb::from_center_extent(xform_col[i].xyz, extent);
            }
        }
    }

    fn rebuild_hash(
        pip_ids: &crate::ecs::class::Class<crate::ecs::PipId>,
        core: &mut <CoreAdd as Addition>::Tables,
        pile: &mut Polypile<dyn AdditionTables>,
        hash: &mut crate::broadphase::SpatialHash,
    ) {
        let Some(broad) = BroadPhaseAdd::tables(pile) else { return };
        hash.clear();

        for (class_id, motion_col) in core.motions.data.iter() {
            if motion_col.key != MotionKind::Active {
                continue;
            }
            let Some(aabb_col) = broad.aabbs.data.get(class_id) else {
                continue;
            };
            let Some(pip_col) = pip_ids.data.get(class_id) else {
                continue;
            };
            for (row_idx, aabb) in aabb_col.iter().enumerate() {
                if let Some(&pip) = pip_col.get(row_idx) {
                    hash.insert(pip, aabb);
                }
            }
        }
    }

    fn generate_pairs(
        ids: &crate::addition::Ids,
        pile: &Polypile<dyn AdditionTables>,
        hash: &crate::broadphase::SpatialHash,
        pairs: &mut crate::broadphase::CandidatePairs,
    ) {
        let Some(broad) = BroadPhaseAdd::tables_ref(pile) else { return };
        pairs.begin_frame(ids.len());

        let aabbs = &broad.aabbs;
        let cells = hash.cells();

        for (_, bucket) in cells {
            for i in 0..bucket.len() {
                for j in (i + 1)..bucket.len() {
                    let a = bucket[i];
                    let b = bucket[j];
                    let Some(aabb_a) = gather_ref(ids, aabbs, a) else { continue };
                    let Some(aabb_b) = gather_ref(ids, aabbs, b) else { continue };
                    if aabb_a.overlaps(&aabb_b) {
                        pairs.try_add(a, b);
                    }
                }
            }
        }
    }
}
