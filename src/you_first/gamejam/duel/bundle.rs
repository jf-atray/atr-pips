//! Entity bundles for the gunslinger duel minigame.

use std::f32::consts::{PI, TAU};

use glam::{Quat, Vec2, Vec3, Vec4};

use crate::anims::*;
use crate::brushes::Brush;
use crate::ecs::{CanvasId, MaterialId};
use crate::ecs::scope::Scope;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::you_first::gamejam::duel::components::{
    DuelCursor, DuelEnemy, DuelReticle, DuelView,
};


pub fn duel_cursor_bundle(
    lateral: f32,
    y: f32,
    size: Vec2,
    color: Vec4,
    canvas: CanvasId,
    mat: MaterialId,
    name: impl Into<String>,
) -> impl FnOnce(&mut Scope) {
    let name = name.into();
    move |scope: &mut Scope| {
        scope.core.with(
            Transform {
                xyz: Vec3::new(lateral, y, 0.2),
                rot: Quat::IDENTITY,
            },
            Brush {
                scale: Vec3::new(size.x, size.y, 1.0),
                color,
                ..Brush::new(canvas, mat)
            },
            name,
            Motion::default(),
        );
        let dv = scope.view::<DuelView>().unwrap();
        dv.duel_cursors = Some(DuelCursor { lateral });
    }
}

pub fn duel_enemy_bundle(
    x: f32,
    y: f32,
    size: Vec2,
    color: Vec4,
    wave: u32,
    speed: f32,
    canvas: CanvasId,
    mat: MaterialId,
    name: impl Into<String>,
) -> impl FnOnce(&mut Scope) {
    let name = name.into();
    move |scope: &mut Scope| {
        scope.core.with(
            Transform {
                xyz: Vec3::new(x, y, 0.2),
                rot: Quat::IDENTITY,
            },
            Brush {
                scale: Vec3::new(size.x, size.y, 1.0),
                color,
                ..Brush::new(canvas, mat)
            },
            name,
            Motion::default(),
        );
        let dv = scope.view::<DuelView>().unwrap();
        dv.duel_enemies = Some(DuelEnemy {
            wave,
            active: true,
            fire_countdown: None,
        });
        dv.duel_reticles = Some(DuelReticle {
            lateral: x,
            d: y,
            speed,
            sway_phase: 0.0,
            snapped: false,
        });
    }
}
const HAT_OFFSET: f32 = 1.5;
const HAT_RISE_HEIGHT: f32 = 1.0;
const HAT_RISE_DUR: f32 = 0.5;
const HAT_FALL_DUR: f32 = 0.5;
const HAT_FALL_RATE: f32 = 3.0;
const HAT_SPIN_DUR: f32 = 0.5;

pub fn build_hat_anim_library() -> (AnimationLibrary, AnimId) {
    let mut lib = AnimationLibrary::default();

    let settle_id = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: 0.0, b: 0.0 }),
            xyz: AnimPair {
                x: None,
                y: Some(AnimFunc::Linear { m: 0.0, b: 0.0 }),
            },
            ..Default::default()
        },
        next: None,
    });

    let fall_id = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: TAU / HAT_SPIN_DUR, b: 0.0 }),
            xyz: AnimPair {
                x: None,
                y: Some(AnimFunc::PowE {
                    m: -HAT_FALL_RATE,
                    b: 0.0,
                    scale: HAT_OFFSET,
                    offs: 0.0,
                }),
            },
            ..Default::default()
        },
        next: Some((HAT_FALL_DUR, settle_id)),
    });

    let rise_id = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: TAU / HAT_SPIN_DUR, b: 0.0 }),
            xyz: AnimPair {
                x: None,
                y: Some(AnimFunc::Sin {
                    m: PI / HAT_RISE_DUR,
                    b: 0.0,
                    scale: HAT_RISE_HEIGHT,
                    offs: HAT_OFFSET,
                }),
            },
            ..Default::default()
        },
        next: Some((HAT_RISE_DUR, fall_id)),
    });

    (lib, rise_id)
}
pub fn build_spin_anim_library() -> (AnimationLibrary, AnimId) {
    let mut lib = AnimationLibrary::default();

    let spin_id = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: 6.0, b: 0.0 }),
            ..Default::default()
        },
        next: None,
    });

    (lib, spin_id)
}

pub fn build_reticle_anim_library() -> (AnimationLibrary, AnimId, AnimId) {
    let mut lib = AnimationLibrary::default();

    let slow_anim = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: 2.0 * PI / 7.8, b: 0.0 }),
            ..Default::default()
        },
        next: None,
    });

    let fast_anim = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: 2.0 * PI / 2.6, b: 0.0 }),
            ..Default::default()
        },
        next: None,
    });

    (lib, slow_anim, fast_anim)
}

pub fn build_living_anim_library() -> (AnimationLibrary, AnimId) {
    let mut lib = AnimationLibrary::default();
    let t = 2.0 / 3.0;
    const SHEER_MAG: f32 = 0.06;
    const SHEER_OFFSET_MAG: f32 = -0.06;
    let hold_1 = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 1.0 }),
                y: Some(AnimFunc::Linear { m: 0.0, b: 1.0 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: SHEER_MAG }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: -SHEER_OFFSET_MAG }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });
    let squash = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::PowE { m: f32::ln(1.1) / t, b: 0.0, scale: 1.0, offs: 0.0 }),
                y: Some(AnimFunc::PowE { m: f32::ln(0.8) / t, b: 0.0, scale: 1.0, offs: 0.0 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: -SHEER_MAG / t, b: SHEER_MAG }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: SHEER_OFFSET_MAG / t, b: -SHEER_OFFSET_MAG }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });
    let hold_2 = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 1.1 }),
                y: Some(AnimFunc::Linear { m: 0.0, b: 0.8 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 0.0 }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 0.0 }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });

    let bounce = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::PowE { m: -f32::ln(1.1) / t, b: 0.0, scale: 1.1, offs: 0.0 }),
                y: Some(AnimFunc::PowE { m: -f32::ln(0.8) / t, b: 0.0, scale: 0.8, offs: 0.0 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: -SHEER_MAG / t, b: 0.0 }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: SHEER_OFFSET_MAG / t, b: 0.0 }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });
    let hold_1b = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 1.0 }),
                y: Some(AnimFunc::Linear { m: 0.0, b: 1.0 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: -SHEER_MAG }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: SHEER_OFFSET_MAG }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });

    let squash_b = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::PowE { m: f32::ln(1.1) / t, b: 0.0, scale: 1.0, offs: 0.0 }),
                y: Some(AnimFunc::PowE { m: f32::ln(0.8) / t, b: 0.0, scale: 1.0, offs: 0.0 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: SHEER_MAG / t, b: -SHEER_MAG }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: -SHEER_OFFSET_MAG / t, b: SHEER_OFFSET_MAG }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });

    let hold_2b = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 1.1 }),
                y: Some(AnimFunc::Linear { m: 0.0, b: 0.8 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 0.0 }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: 0.0, b: 0.0 }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });
    let bounce_b = lib.insert(AnimSequence {
        rules: AnimRules {
            scale: AnimPair {
                x: Some(AnimFunc::PowE { m: -f32::ln(1.1) / t, b: 0.0, scale: 1.1, offs: 0.0 }),
                y: Some(AnimFunc::PowE { m: -f32::ln(0.8) / t, b: 0.0, scale: 0.8, offs: 0.0 }),
            },
            sheer: AnimPair {
                x: Some(AnimFunc::Linear { m: SHEER_MAG / t, b: 0.0 }),
                y: None,
            },
            xyz: AnimPair {
                x: Some(AnimFunc::Linear { m: -SHEER_OFFSET_MAG / t, b: 0.0 }),
                y: None,
            },
            ..Default::default()
        },
        next: Some((t, hold_1)),
    });
    lib.get_mut(hold_1).unwrap().next = Some((t, squash));
    lib.get_mut(squash).unwrap().next = Some((t, hold_2));
    lib.get_mut(hold_2).unwrap().next = Some((t, bounce));
    lib.get_mut(bounce).unwrap().next = Some((t, hold_1b));
    lib.get_mut(hold_1b).unwrap().next = Some((t, squash_b));
    lib.get_mut(squash_b).unwrap().next = Some((t, hold_2b));
    lib.get_mut(hold_2b).unwrap().next = Some((t, bounce_b));

    (lib, hold_1)
}

pub fn build_tumbleweed_anim_library() -> (AnimationLibrary, AnimId) {
    let mut lib = AnimationLibrary::default();
    let t = 2.0;
    let tau = 2.0 * PI;

    let bounce = lib.insert(AnimSequence {
        rules: AnimRules {
            spin: Some(AnimFunc::Linear { m: -tau / t, b: 0.0 }),
            xyz: AnimPair {
                y: Some(AnimFunc::Sin {
                    m: PI / t,
                    b: 0.0,
                    scale: 0.88,
                    offs: 0.0,
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        next: Some((t, AnimId::default())),
    });

    lib.get_mut(bounce).unwrap().next = Some((t, bounce));

    (lib, bounce)
}
