use glam::{Quat, Vec2, Vec3};

use crate::collision::contact::{ContactCache, ContactPair, ManifoldPoint};
use crate::ecs::gather::impls::{gather_ref, gather_two_pair_mut};
use crate::ecs::PipId;
use crate::spacial::motion::{Motion, MotionKind};
use crate::spacial::transform::Transform;

pub fn solve_contacts(
    cache: &mut ContactCache,
    ids: &slotmap::SlotMap<PipId, crate::ecs::ClassRowPtr>,
    inv_masses: &crate::ecs::class::Class<f32>,
    inv_inertias: &crate::ecs::class::Class<f32>,
    motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    xforms: &mut crate::ecs::class::Class<Transform>,
    iterations: u32,
) {
    for _ in 0..iterations {
        for i in 0..cache.len() {
            let pair = &mut cache.pairs_mut()[i];

            let Some((motion_a, xform_a, motion_b, xform_b)) =
                gather_two_pair_mut(ids, motions, xforms, pair.body_a, pair.body_b)
            else { continue };

            let inv_mass_a = *gather_ref(ids, inv_masses, pair.body_a).unwrap_or(&0.0);
            let inv_inertia_a = *gather_ref(ids, inv_inertias, pair.body_a).unwrap_or(&0.0);
            let inv_mass_b = *gather_ref(ids, inv_masses, pair.body_b).unwrap_or(&0.0);
            let inv_inertia_b = *gather_ref(ids, inv_inertias, pair.body_b).unwrap_or(&0.0);

            solve_pair(
                pair,
                motion_a, xform_a, inv_mass_a, inv_inertia_a,
                motion_b, xform_b, inv_mass_b, inv_inertia_b,
            );
        }
    }
}

fn solve_pair(
    pair: &mut ContactPair,
    motion_a: &mut Motion, xform_a: &Transform, inv_mass_a: f32, inv_inertia_a: f32,
    motion_b: &mut Motion, xform_b: &Transform, inv_mass_b: f32, inv_inertia_b: f32,
) {
    let n = Vec2::new(pair.normal.x, pair.normal.y);
    let pos_a = xform_a.xyz;
    let pos_b = xform_b.xyz;
    let rot_a = xform_a.rot;
    let rot_b = xform_b.rot;

    for i in 0..pair.point_count as usize {
        let point = &mut pair.points[i];
        let r_a = world_offset_a(point, pos_a, rot_a);
        let r_b = world_offset_b(point, pos_b, rot_b);

        let vel_a = motion_a.vel + cross_scalar(motion_a.ang_vel, r_a);
        let vel_b = motion_b.vel + cross_scalar(motion_b.ang_vel, r_b);
        let rel_vel = vel_b - vel_a;
        let vn = rel_vel.x * n.x + rel_vel.y * n.y;

        let ra_cross_n = r_a.x * n.y - r_a.y * n.x;
        let rb_cross_n = r_b.x * n.y - r_b.y * n.x;
        let inv_mass_eff = inv_mass_a + inv_mass_b
            + ra_cross_n * ra_cross_n * inv_inertia_a
            + rb_cross_n * rb_cross_n * inv_inertia_b;

        if inv_mass_eff <= 0.0 {
            continue;
        }

        let restitution_bias = if point.separation > -0.005 {
            pair.restitution * vn.min(0.0)
        } else {
            0.0
        };

        let j = -(vn - restitution_bias) / inv_mass_eff;
        let mut new_impulse = point.normal_impulse + j;
        if new_impulse < 0.0 {
            new_impulse = 0.0;
        }
        let j_clamped = new_impulse - point.normal_impulse;
        point.normal_impulse = new_impulse;

        let impulse = Vec3::new(n.x * j_clamped, n.y * j_clamped, 0.0);
        apply_impulse(motion_a, -impulse, r_a, inv_mass_a, inv_inertia_a);
        apply_impulse(motion_b, impulse, r_b, inv_mass_b, inv_inertia_b);

        let vel_a = motion_a.vel + cross_scalar(motion_a.ang_vel, r_a);
        let vel_b = motion_b.vel + cross_scalar(motion_b.ang_vel, r_b);
        let rel_vel = vel_b - vel_a;
        let vn_new = rel_vel.x * n.x + rel_vel.y * n.y;
        let vt = Vec2::new(rel_vel.x, rel_vel.y) - n * vn_new;
        let vt_len = vt.length();
        if vt_len < 1e-6 {
            continue;
        }
        let t = vt / vt_len;

        let ra_cross_t = r_a.x * t.y - r_a.y * t.x;
        let rb_cross_t = r_b.x * t.y - r_b.y * t.x;
        let inv_mass_t = inv_mass_a + inv_mass_b
            + ra_cross_t * ra_cross_t * inv_inertia_a
            + rb_cross_t * rb_cross_t * inv_inertia_b;

        if inv_mass_t <= 0.0 {
            continue;
        }

        let jt = -vt_len / inv_mass_t;
        let max_friction = pair.friction * point.normal_impulse;
        let mut new_tangent = point.tangent_impulse + jt;
        new_tangent = new_tangent.clamp(-max_friction, max_friction);
        let jt_clamped = new_tangent - point.tangent_impulse;
        point.tangent_impulse = new_tangent;

        let tangent_impulse = Vec3::new(t.x * jt_clamped, t.y * jt_clamped, 0.0);
        apply_impulse(motion_a, -tangent_impulse, r_a, inv_mass_a, inv_inertia_a);
        apply_impulse(motion_b, tangent_impulse, r_b, inv_mass_b, inv_inertia_b);
    }
}

fn apply_impulse(motion: &mut Motion, impulse: Vec3, r: Vec2, inv_mass: f32, inv_inertia: f32) {
    motion.vel += impulse * inv_mass;
    let torque = r.x * impulse.y - r.y * impulse.x;
    motion.ang_vel += torque * inv_inertia;
}

fn world_offset_a(point: &ManifoldPoint, pos: Vec3, rot: Quat) -> Vec2 {
    let world = pos + rot * point.local_a;
    Vec2::new(world.x - pos.x, world.y - pos.y)
}

fn world_offset_b(point: &ManifoldPoint, pos: Vec3, rot: Quat) -> Vec2 {
    let world = pos + rot * point.local_b;
    Vec2::new(world.x - pos.x, world.y - pos.y)
}

fn cross_scalar(s: f32, v: Vec2) -> Vec3 {
    Vec3::new(-s * v.y, s * v.x, 0.0)
}
