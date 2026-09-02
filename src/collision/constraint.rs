use glam::{Quat, Vec2, Vec3};

use crate::collision::contact::{ContactCache, ContactPair, ManifoldPoint};
use crate::ecs::gather::impls::{gather_ref, gather_two_mut, gather_two_pair_mut};
use crate::ecs::PipId;
use crate::spacial::motion::{Motion, MotionKind};
use crate::spacial::transform::Transform;

const SLOP: f32 = 0.005;
const BAUMGARTE: f32 = 0.2;
const MAX_CORRECTION: f32 = 0.2;

pub fn solve_contacts(
    cache: &mut ContactCache,
    ids: &slotmap::SlotMap<PipId, crate::ecs::ClassRowPtr>,
    inv_masses: &crate::ecs::class::Class<f32>,
    inv_inertias: &crate::ecs::class::Class<f32>,
    motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    xforms: &mut crate::ecs::class::Class<Transform>,
    gravity: Vec3,
    drag: f32,
    velocity_iterations: u32,
    position_iterations: u32,
    dt: f32,
) {
    integrate_damping(motions, drag, dt);
    integrate_gravity(motions, inv_masses, gravity);
    warm_start(cache, ids, inv_masses, inv_inertias, motions, xforms);

    for _ in 0..velocity_iterations {
        solve_velocity(cache, ids, inv_masses, inv_inertias, motions, xforms);
    }

    integrate_positions(motions, xforms, dt);

    for _ in 0..position_iterations {
        solve_position(cache, ids, inv_masses, inv_inertias, xforms);
    }
}

fn integrate_damping(
    mut motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    drag: f32,
    dt: f32,
) {
    if drag <= 0.0 {
        return;
    }
    let decay = (1.0 - drag).powf(dt);
    crate::query!(
        [MotionKind::Active; &mut motions],
        |motion| {
            motion.vel *= decay;
            motion.ang_vel *= decay;
        }
    );
}

fn integrate_gravity(
    mut motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    inv_masses: &crate::ecs::class::Class<f32>,
    gravity: Vec3,
) {
    crate::query!(
        [MotionKind::Active; &mut motions, (); &inv_masses],
        |motion, inv_mass| {
            if inv_mass.is_normal() {
                motion.vel += gravity;
            }
        }
    );
}

fn warm_start(
    cache: &mut ContactCache,
    ids: &slotmap::SlotMap<PipId, crate::ecs::ClassRowPtr>,
    inv_masses: &crate::ecs::class::Class<f32>,
    inv_inertias: &crate::ecs::class::Class<f32>,
    motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    xforms: &mut crate::ecs::class::Class<Transform>,
) {
    for i in 0..cache.len() {
        let pair = &cache.pairs()[i];
        let n = Vec2::new(pair.normal.x, pair.normal.y);

        let inv_mass_a = *gather_ref(ids, inv_masses, pair.body_a).unwrap_or(&0.0);
        let inv_inertia_a = *gather_ref(ids, inv_inertias, pair.body_a).unwrap_or(&0.0);
        let inv_mass_b = *gather_ref(ids, inv_masses, pair.body_b).unwrap_or(&0.0);
        let inv_inertia_b = *gather_ref(ids, inv_inertias, pair.body_b).unwrap_or(&0.0);

        let Some((motion_a, xform_a, motion_b, xform_b)) =
            gather_two_pair_mut(ids, motions, xforms, pair.body_a, pair.body_b)
        else { continue };

        let pos_a = xform_a.xyz;
        let pos_b = xform_b.xyz;
        let rot_a = xform_a.rot;
        let rot_b = xform_b.rot;

        for j in 0..pair.point_count as usize {
            let point = &pair.points[j];
            let r_a = world_offset_a(point, pos_a, rot_a);
            let r_b = world_offset_b(point, pos_b, rot_b);

            let normal_impulse = Vec3::new(n.x * point.normal_impulse, n.y * point.normal_impulse, 0.0);
            apply_impulse(motion_a, -normal_impulse, r_a, inv_mass_a, inv_inertia_a);
            apply_impulse(motion_b, normal_impulse, r_b, inv_mass_b, inv_inertia_b);

            let t = Vec2::new(-n.y, n.x);
            let tangent_impulse = Vec3::new(t.x * point.tangent_impulse, t.y * point.tangent_impulse, 0.0);
            apply_impulse(motion_a, -tangent_impulse, r_a, inv_mass_a, inv_inertia_a);
            apply_impulse(motion_b, tangent_impulse, r_b, inv_mass_b, inv_inertia_b);
        }
    }
}

fn solve_velocity(
    cache: &mut ContactCache,
    ids: &slotmap::SlotMap<PipId, crate::ecs::ClassRowPtr>,
    inv_masses: &crate::ecs::class::Class<f32>,
    inv_inertias: &crate::ecs::class::Class<f32>,
    motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    xforms: &mut crate::ecs::class::Class<Transform>,
) {
    for i in 0..cache.len() {
        let pair = &mut cache.pairs_mut()[i];

        let inv_mass_a = *gather_ref(ids, inv_masses, pair.body_a).unwrap_or(&0.0);
        let inv_inertia_a = *gather_ref(ids, inv_inertias, pair.body_a).unwrap_or(&0.0);
        let inv_mass_b = *gather_ref(ids, inv_masses, pair.body_b).unwrap_or(&0.0);
        let inv_inertia_b = *gather_ref(ids, inv_inertias, pair.body_b).unwrap_or(&0.0);

        let Some((motion_a, xform_a, motion_b, xform_b)) =
            gather_two_pair_mut(ids, motions, xforms, pair.body_a, pair.body_b)
        else { continue };

        solve_pair_velocity(
            pair,
            motion_a, xform_a, inv_mass_a, inv_inertia_a,
            motion_b, xform_b, inv_mass_b, inv_inertia_b,
        );
    }
}

fn solve_position(
    cache: &mut ContactCache,
    ids: &slotmap::SlotMap<PipId, crate::ecs::ClassRowPtr>,
    inv_masses: &crate::ecs::class::Class<f32>,
    _inv_inertias: &crate::ecs::class::Class<f32>,
    xforms: &mut crate::ecs::class::Class<Transform>,
) {
    for i in 0..cache.len() {
        let pair = &cache.pairs()[i];
        let n = Vec2::new(pair.normal.x, pair.normal.y);

        let Some((xform_a, xform_b)) =
            gather_two_mut(ids, xforms, pair.body_a, pair.body_b)
        else { continue };

        let inv_mass_a = *gather_ref(ids, inv_masses, pair.body_a).unwrap_or(&0.0);
        let inv_mass_b = *gather_ref(ids, inv_masses, pair.body_b).unwrap_or(&0.0);

        let inv_mass_sum = inv_mass_a + inv_mass_b;
        if inv_mass_sum <= 0.0 {
            continue;
        }

        let pos_a = xform_a.xyz;
        let pos_b = xform_b.xyz;
        let rot_a = xform_a.rot;
        let rot_b = xform_b.rot;

        let mut min_separation = f32::MAX;
        for j in 0..pair.point_count as usize {
            let point = &pair.points[j];
            let world_a = pos_a + rot_a * point.local_a;
            let world_b = pos_b + rot_b * point.local_b;
            let delta = world_b - world_a;
            let sep = delta.x * n.x + delta.y * n.y;
            if sep < min_separation {
                min_separation = sep;
            }
        }

        if min_separation >= -SLOP {
            continue;
        }

        let correction = ((-min_separation - SLOP) * BAUMGARTE).min(MAX_CORRECTION);
        let impulse = Vec3::new(n.x * correction, n.y * correction, 0.0);
        xform_a.xyz -= impulse * inv_mass_a / inv_mass_sum;
        xform_b.xyz += impulse * inv_mass_b / inv_mass_sum;
    }
}

fn integrate_positions(
    motions: &mut crate::ecs::class::Class<Motion, MotionKind>,
    xforms: &mut crate::ecs::class::Class<Transform>,
    dt: f32,
) {
    for (class_id, motion_col) in motions.data.iter_mut() {
        if motion_col.key != MotionKind::Active {
            continue;
        }
        let Some(xform_col) = xforms.data.get_mut(class_id) else { continue };
        for i in 0..motion_col.len() {
            xform_col[i].xyz += motion_col[i].vel * dt;
            xform_col[i].rot = Quat::from_rotation_z(motion_col[i].ang_vel * dt) * xform_col[i].rot;
        }
    }
}

fn solve_pair_velocity(
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

        if !inv_mass_eff.is_normal() {
            continue;
        }

        let restitution_bias = if point.separation > -SLOP {
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

        let t = Vec2::new(-n.y, n.x);

        let vel_a = motion_a.vel + cross_scalar(motion_a.ang_vel, r_a);
        let vel_b = motion_b.vel + cross_scalar(motion_b.ang_vel, r_b);
        let rel_vel = vel_b - vel_a;
        let vt = rel_vel.x * t.x + rel_vel.y * t.y;

        let ra_cross_t = r_a.x * t.y - r_a.y * t.x;
        let rb_cross_t = r_b.x * t.y - r_b.y * t.x;
        let inv_mass_t = inv_mass_a + inv_mass_b
            + ra_cross_t * ra_cross_t * inv_inertia_a
            + rb_cross_t * rb_cross_t * inv_inertia_b;

        if !inv_mass_t.is_normal() {
            continue;
        }

        let jt = -vt / inv_mass_t;
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

pub(crate) fn world_offset_a(point: &ManifoldPoint, pos: Vec3, rot: Quat) -> Vec2 {
    let world = pos + rot * point.local_a;
    Vec2::new(world.x - pos.x, world.y - pos.y)
}

pub(crate) fn world_offset_b(point: &ManifoldPoint, pos: Vec3, rot: Quat) -> Vec2 {
    let world = pos + rot * point.local_b;
    Vec2::new(world.x - pos.x, world.y - pos.y)
}

pub(crate) fn cross_scalar(s: f32, v: Vec2) -> Vec3 {
    Vec3::new(-s * v.y, s * v.x, 0.0)
}
