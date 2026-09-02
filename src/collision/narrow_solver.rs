use std::collections::HashMap;

use glam::{Quat, Vec3};

use crate::addition::{Addition, Pips, Polypile, ScriptsMap, SignalsMap, Solver, Tables as AdditionTables};
use crate::assets::SpriteEntry;
use crate::collision::constraint::solve_contacts;
use crate::collision::CollisionAdd;
use crate::ecs::core::CoreAdd;
use crate::ecs::gather::impls::gather_ref;
use crate::ecs::PipId;
use crate::gather::impls::gather_pair_ref;
use crate::input::Input;
use crate::collision::contact::{ContactCache, ContactPair, ManifoldPoint};
use crate::collision::obb::{clip, sat, world_to_local, ClipPoint, Obb};
use crate::physics::data::material::Material;
use crate::physics::PhysicsAdd;

#[derive(Debug)]
pub struct NarrowPhaseSolver {
    cache: ContactCache,
}

impl Solver for NarrowPhaseSolver {}

impl NarrowPhaseSolver {
    pub fn new() -> Self {
        Self {
            cache: ContactCache::default(),
        }
    }

    pub fn cache(&self) -> &[ContactPair] {
        self.cache.pairs()
    }

    #[allow(clippy::unused_self, reason = "cache is on self")]
    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        _input: &mut Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let Some(broad) = CollisionAdd::signals_ref(signals) else {
            return;
        };
        let candidate_pairs = &broad.pairs.pairs;

        self.cache.begin_frame();

        for &(a, b) in candidate_pairs {
            let Some((obb_a, mat_a, pos_a, rot_a)) = Self::gather_body(&pips.ids, &pips.tables.core, &pips.tables.pile, a) else { continue };
            let Some((obb_b, mat_b, pos_b, rot_b)) = Self::gather_body(&pips.ids, &pips.tables.core, &pips.tables.pile, b) else { continue };

            let Some(result) = sat(&obb_a, &obb_b) else { continue };
            let clip_points = clip(&obb_a, &obb_b, &result);

            let point_count = clip_points.iter().flatten().count();
            if point_count == 0 {
                continue;
            }

            let friction = (mat_a.friction * mat_b.friction).sqrt();
            let restitution = mat_a.restitution.max(mat_b.restitution);
            let normal = Vec3::new(result.normal.x, result.normal.y, 0.0);

            let key = (a, b);
            match self.cache.find(key) {
                Ok(index) => Self::update_pair(&mut self.cache, index, clip_points, &result, normal, friction, restitution, pos_a, rot_a, pos_b, rot_b),
                Err(index) => Self::insert_pair(&mut self.cache, index, a, b, clip_points, &result, normal, friction, restitution, pos_a, rot_a, pos_b, rot_b),
            }
        }

        self.cache.evict_untouched();

        let core = &mut pips.tables.core;
        let Some(physics) = PhysicsAdd::tables(&mut pips.tables.pile) else { return };
        solve_contacts(
            &mut self.cache,
            &pips.ids,
            &physics.inv_masses,
            &physics.inv_inertias,
            &mut core.motions,
            &mut core.xforms,
            8,
        );
    }

    fn update_pair(
        cache: &mut ContactCache,
        index: usize,
        clip_points: [Option<ClipPoint>; 2],
        result: &crate::collision::obb::SatResult,
        normal: Vec3,
        friction: f32,
        restitution: f32,
        pos_a: Vec3,
        rot_a: Quat,
        pos_b: Vec3,
        rot_b: Quat,
    ) {
        let pair = cache.get_mut(index);
        pair.touched = true;
        pair.normal = normal;
        pair.friction = friction;
        pair.restitution = restitution;

        let old_points = pair.active_points();

        let mut new_points = [ManifoldPoint::default(); 2];
        let mut count = 0;

        for cp in clip_points.iter().flatten() {
            if count >= 2 {
                break;
            }
            let local_a = world_to_local(cp.world, pos_a, rot_a);
            let local_b = world_to_local(cp.world, pos_b, rot_b);
            let contact_id = make_contact_id(result.ref_on_a, result.ref_edge, cp.inc_edge, cp.inc_vertex);

            // Warm start: copy impulses from the matching old point
            let (normal_impulse, tangent_impulse) = old_points
                .iter()
                .find(|op| op.contact_id == contact_id)
                .map(|op| (op.normal_impulse, op.tangent_impulse))
                .unwrap_or((0.0, 0.0));

            new_points[count] = ManifoldPoint {
                local_a,
                local_b,
                separation: cp.separation,
                contact_id,
                normal_impulse,
                tangent_impulse,
            };
            count += 1;
        }

        pair.points = new_points;
        pair.point_count = count as u8;
    }

    fn insert_pair(
        cache: &mut ContactCache,
        index: usize,
        a: PipId,
        b: PipId,
        clip_points: [Option<ClipPoint>; 2],
        result: &crate::collision::obb::SatResult,
        normal: Vec3,
        friction: f32,
        restitution: f32,
        pos_a: Vec3,
        rot_a: Quat,
        pos_b: Vec3,
        rot_b: Quat,
    ) {
        let mut points = [ManifoldPoint::default(); 2];
        let mut count = 0;

        for cp in clip_points.iter().flatten() {
            if count >= 2 {
                break;
            }
            let local_a = world_to_local(cp.world, pos_a, rot_a);
            let local_b = world_to_local(cp.world, pos_b, rot_b);
            let contact_id = make_contact_id(result.ref_on_a, result.ref_edge, cp.inc_edge, cp.inc_vertex);

            points[count] = ManifoldPoint {
                local_a,
                local_b,
                separation: cp.separation,
                contact_id,
                normal_impulse: 0.0,
                tangent_impulse: 0.0,
            };
            count += 1;
        }

        cache.insert(index, ContactPair {
            body_a: a,
            body_b: b,
            normal,
            friction,
            restitution,
            points,
            point_count: count as u8,
            touched: true,
        });
    }

    fn gather_body(
        ids: &crate::addition::Ids,
        core: &<CoreAdd as Addition>::Tables,
        pile: &Polypile<dyn AdditionTables>,
        pip: PipId,
    ) -> Option<(Obb, Material, Vec3, Quat)> {
        let (xform, brush) = gather_pair_ref(ids, &core.xforms, &core.brushes, pip)?;
        let Some(physics) = PhysicsAdd::tables_ref(pile) else { return None };
        let material = gather_ref(ids, &physics.materials, pip)?;
        let obb = Obb::from_transform(xform.xyz, xform.rot, brush.scale);
        Some((obb, *material, xform.xyz, xform.rot))
    }
}

fn make_contact_id(ref_on_a: bool, ref_edge: usize, inc_edge: usize, inc_vertex: usize) -> u16 {
    ((ref_on_a as u16) << 12) | ((ref_edge as u16) << 8) | ((inc_edge as u16) << 4) | (inc_vertex as u16)
}

