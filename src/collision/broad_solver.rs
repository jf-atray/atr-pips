use std::collections::HashMap;

use crate::addition::{Addition, Pips, Polypile, ScriptsMap, SignalsMap, Solver, Tables as AdditionTables};
use crate::assets::SpriteEntry;
use crate::collision::CollisionAdd;
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
        let Some(broad_signals) = CollisionAdd::signals(signals) else {
            return;
        };

        Self::update_aabbs(&mut pips.tables.core, &mut pips.tables.pile);
        Self::rebuild_hash(&mut pips.pip_ids, &mut pips.tables.core, &mut pips.tables.pile, &mut broad_signals.hash);
        Self::generate_pairs(&pips.ids, &pips.tables.pile, &broad_signals.hash, &mut broad_signals.pairs);
    }

    fn update_aabbs(
        core: &mut <CoreAdd as Addition>::Tables,
        pile: &mut Polypile<dyn AdditionTables>,
    ) {
        let Some(broad) = CollisionAdd::tables(pile) else { return };

        crate::query!(
            [MotionKind::Active; &mut core.motions, (); &mut core.xforms, (); &mut core.brushes, (); &mut broad.aabbs],
            |_, xform, brush, aabb| {
                let extent = brush.scale * 0.5;
                *aabb = Aabb::from_center_extent(xform.xyz, extent);
            }
        );
    }

    fn rebuild_hash(
        mut pip_ids: &mut crate::ecs::class::Class<crate::ecs::PipId>,
        core: &mut <CoreAdd as Addition>::Tables,
        pile: &mut Polypile<dyn AdditionTables>,
        hash: &mut crate::collision::SpatialHash,
    ) {
        let Some(broad) = CollisionAdd::tables(pile) else { return };
        hash.clear();

        crate::query!(
            [MotionKind::Active; &mut core.motions, (); &mut broad.aabbs, (); &mut pip_ids],
            |_, aabb, pip_id| {
                hash.insert(*pip_id, aabb);
            }
        );
    }

    fn generate_pairs(
        ids: &crate::addition::Ids,
        pile: &Polypile<dyn AdditionTables>,
        hash: &crate::collision::SpatialHash,
        pairs: &mut crate::collision::CandidatePairs,
    ) {
        let Some(broad) = CollisionAdd::tables_ref(pile) else { return };
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


