use glam::Quat;
use slotmap::SlotMap;

use crate::brushes::Brush;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use crate::ecs::{CanvasId, MaterialId};
use crate::spacial::transform::Transform;

pub mod solver;

#[derive(Debug, Clone, Copy, Default)]
pub struct AnimTime(pub f32);

impl AnimTime {
    pub fn get_or_zero(self) -> f32 {
        if self.0.is_finite() { self.0 } else { 0.0 }
    }

    pub fn reset_if_nan(&mut self) -> bool {
        if self.0.is_finite() {
            false
        } else {
            self.0 = 0.0;
            true
        }
    }

    pub fn snap_to_beat(&mut self, master_clock: f32, beat: f32, tolerance: f32) {
        if !self.0.is_finite() || beat <= 0.0 {
            return;
        }
        let phase_error = (self.0 - master_clock).rem_euclid(beat);
        let phase_error = if phase_error > beat * 0.5 {
            phase_error - beat
        } else {
            phase_error
        };
        if phase_error.abs() > tolerance {
            self.0 -= phase_error;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimSpin {
    pub func: Option<AnimFunc>,
    pub rot_base: f32,
}

#[derive(Debug, Clone, Default)]
pub struct AnimScale(pub AnimPair);

#[derive(Debug, Clone, Default)]
pub struct AnimXyz(pub AnimPair);

#[derive(Debug, Clone, Default)]
pub struct AnimSheer(pub AnimPair);

#[derive(Debug, Clone, Default)]
pub struct AnimSprite(pub Option<(CanvasId, MaterialId)>);

#[derive(Debug, Clone, Default)]
pub struct AnimKeyframe {
    pub id: AnimId,
    pub lib: AnimLibId,
}

#[derive(Default, Clone, Debug)]
pub struct AnimRules {
    pub xyz: AnimPair,
    pub scale: AnimPair,
    pub sheer: AnimPair,
    pub spin: Option<AnimFunc>,
    pub sprite: Option<(CanvasId, MaterialId)>,
}

#[derive(Default, Debug, Clone)]
pub struct AnimSequence {
    pub rules: AnimRules,
    pub next: Option<(f32, AnimId)>,
}

slotmap::new_key_type! {
    pub struct AnimId;
    pub struct AnimLibId;
}

#[derive(Default, Debug, Clone)]
pub struct AnimationLibrary {
    anims: SlotMap<AnimId, AnimSequence>,
}

impl AnimationLibrary {
    pub fn get(&self, id: AnimId) -> Option<&AnimSequence> {
        self.anims.get(id)
    }

    pub fn get_mut(&mut self, id: AnimId) -> Option<&mut AnimSequence> {
        self.anims.get_mut(id)
    }

    pub fn insert(&mut self, seq: AnimSequence) -> AnimId {
        self.anims.insert(seq)
    }

    pub fn is_in_chain(&self, root: AnimId, candidate: AnimId) -> bool {
        if root == candidate {
            return true;
        }
        let mut current = root;
        for _ in 0..64 {
            let Some(seq) = self.anims.get(current) else {
                break;
            };
            let Some((_, next_id)) = seq.next else { break };
            if next_id == candidate {
                return true;
            }
            if next_id == root {
                break;
            }
            current = next_id;
        }
        false
    }
}

#[derive(Default, Clone, Debug)]
pub struct AnimPair {
    pub x: Option<AnimFunc>,
    pub y: Option<AnimFunc>,
}

#[derive(Clone, Debug)]
pub enum AnimFunc {
    Sin {
        m: f32,
        b: f32,
        scale: f32,
        offs: f32,
    },
    Linear {
        m: f32,
        b: f32,
    },
    Ln {
        m: f32,
        b: f32,
        scale: f32,
        offs: f32,
    },
    PowE {
        m: f32,
        b: f32,
        scale: f32,
        offs: f32,
    },
}

impl AnimFunc {
    pub fn solve(&self, x: f32) -> f32 {
        match self {
            AnimFunc::Sin { m, b, scale, offs } => (((x * m) + b).sin() * scale) + offs,
            AnimFunc::Linear { m, b } => m * x + b,
            AnimFunc::Ln { m, b, scale, offs } => (f32::ln((x * m) + b) * scale) + offs,
            AnimFunc::PowE { m, b, scale, offs } => (f32::exp((x * m) + b) * scale) + offs,
        }
    }
}

pub fn solve_spin(time: &mut AnimTime, spin: &AnimSpin, transform: &mut Transform) {
    let t = time.get_or_zero();
    if let Some(f) = &spin.func {
        transform.rot = Quat::from_rotation_z(spin.rot_base + f.solve(t));
    } else {
        transform.rot = Quat::from_rotation_z(spin.rot_base);
    }
}

pub fn solve_xyz(time: &AnimTime, xyz: &AnimXyz, brush: &mut Brush) {
    let t = time.get_or_zero();
    if let Some(f) = &xyz.0.x {
        brush.offset.x = f.solve(t);
    }
    if let Some(f) = &xyz.0.y {
        brush.offset.y = f.solve(t);
    }
}

pub fn solve_scale(time: &AnimTime, scale: &AnimScale, brush: &mut Brush) {
    let t = time.get_or_zero();
    if let Some(f) = &scale.0.x {
        brush.scale.x *= f.solve(t);
    }
    if let Some(f) = &scale.0.y {
        brush.scale.y *= f.solve(t);
    }
}

pub fn solve_sheer(time: &AnimTime, sheer: &AnimSheer, brush: &mut Brush) {
    let t = time.get_or_zero();
    if let Some(f) = &sheer.0.x {
        brush.sheer.x = f.solve(t);
    }
    if let Some(f) = &sheer.0.y {
        brush.sheer.y = f.solve(t);
    }
}

pub fn solve_sprite(sprite: &AnimSprite, brush: &mut Brush) {
    if let Some((canvas, material)) = sprite.0 {
        brush.canvas = canvas;
        brush.material = material;
    }
}

pub fn advance(
    anim_time: &mut AnimTime,
    keyframe: &mut AnimKeyframe,
    lib: &AnimationLibrary,
) -> Option<AnimId> {
    if let Some(seq) = lib.get(keyframe.id)
        && let Some((threshold, next_id)) = seq.next
        && anim_time.0 >= threshold
    {
        anim_time.0 = 0.0;
        keyframe.id = next_id;
        return Some(next_id);
    }
    None
}

pub fn refresh(anim_time: &mut AnimTime, keyframe: &AnimKeyframe, lib: &AnimationLibrary) -> bool {
    if anim_time.reset_if_nan() {
        lib.get(keyframe.id).is_some()
    } else {
        false
    }
}

crate::addition! {
    #[derive(Debug)]
    pub struct anim_world : AnimWorld {
        tables: {
            anim_times: Class<AnimTime> = Class::new(GrowthStrategy::quart_kib::<AnimTime>()),
            anim_keyframes: Class<AnimKeyframe> = Class::new(GrowthStrategy::quart_kib::<AnimKeyframe>()),
            anim_spins: Class<AnimSpin> = Class::new(GrowthStrategy::quart_kib::<AnimSpin>()),
            anim_scales: Class<AnimScale> = Class::new(GrowthStrategy::quart_kib::<AnimScale>()),
            anim_xyzs: Class<AnimXyz> = Class::new(GrowthStrategy::quart_kib::<AnimXyz>()),
            anim_sheers: Class<AnimSheer> = Class::new(GrowthStrategy::quart_kib::<AnimSheer>()),
            anim_sprites: Class<AnimSprite> = Class::new(GrowthStrategy::quart_kib::<AnimSprite>()),
        },
        solvers: { anim_solver: crate::anims::solver::AnimSolver = crate::anims::solver::AnimSolver },
        scripts: {},
        signals: {},
    }
}
