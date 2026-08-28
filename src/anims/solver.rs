use std::collections::HashMap;

use crate::addition::{Addition, Pips, ScriptsMap, SignalsMap, Solver};
use crate::anims::{
    advance, refresh, solve_scale, solve_sheer, solve_spin, solve_sprite, solve_xyz,
    AnimAdd,
};
use crate::assets::SpriteEntry;
use crate::input::Input;
use crate::query;

#[derive(Debug)]
pub struct AnimSolver;

impl Solver for AnimSolver {}

impl AnimSolver {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::unused_self, reason = "called via solver dispatch")]
    pub fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        _signals: &mut SignalsMap,
        _input: &Input,
        _asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        let libs = &pips.anim_libs;
        let core = &mut pips.tables.core;

        let Some(anim) = AnimAdd::tables(&mut pips.tables.pile) else {
            return;
        };

        query!(
            [&mut anim.anim_times, &mut anim.anim_keyframes],
            |time, keyframe| {
                if let Some(lib) = libs.get(keyframe.lib) {
                    let _ = refresh(time, keyframe, lib);
                    time.0 += dt;
                    let _ = advance(time, keyframe, lib);
                } else {
                    time.0 += dt;
                }
            }
        );

        query!(
            [&mut anim.anim_keyframes, &mut anim.anim_spins],
            |keyframe, spin| {
                if let Some(lib) = libs.get(keyframe.lib)
                    && let Some(seq) = lib.get(keyframe.id)
                {
                    let rules = &seq.rules;
                    spin.func.clone_from(&rules.spin);
                }
            }
        );

        query!(
            [&mut anim.anim_keyframes, &mut anim.anim_scales],
            |keyframe, scale| {
                if let Some(lib) = libs.get(keyframe.lib)
                    && let Some(seq) = lib.get(keyframe.id)
                {
                    let rules = &seq.rules;
                    scale.0 = rules.scale.clone();
                }
            }
        );

        query!(
            [&mut anim.anim_keyframes, &mut anim.anim_xyzs],
            |keyframe, xyz| {
                if let Some(lib) = libs.get(keyframe.lib)
                    && let Some(seq) = lib.get(keyframe.id)
                {
                    let rules = &seq.rules;
                    xyz.0 = rules.xyz.clone();
                }
            }
        );

        query!(
            [&mut anim.anim_keyframes, &mut anim.anim_sheers],
            |keyframe, sheer| {
                if let Some(lib) = libs.get(keyframe.lib)
                    && let Some(seq) = lib.get(keyframe.id)
                {
                    let rules = &seq.rules;
                    sheer.0 = rules.sheer.clone();
                }
            }
        );

        query!(
            [&mut anim.anim_keyframes, &mut anim.anim_sprites],
            |keyframe, sprite| {
                if let Some(lib) = libs.get(keyframe.lib)
                    && let Some(seq) = lib.get(keyframe.id)
                {
                    let rules = &seq.rules;
                    sprite.0 = rules.sprite;
                }
            }
        );

        query!(
            [&mut anim.anim_times, &mut anim.anim_spins, &mut core.xforms],
            |time, spin, xform| {
                solve_spin(time, spin, xform);
            }
        );

        query!(
            [&mut anim.anim_times, &mut anim.anim_xyzs, &mut core.brushes],
            |time, xyz, brush| {
                solve_xyz(*time, xyz, brush);
            }
        );

        query!(
            [&mut anim.anim_times, &mut anim.anim_scales, &mut core.brushes],
            |time, scale, brush| {
                solve_scale(*time, scale, brush);
            }
        );

        query!(
            [&mut anim.anim_times, &mut anim.anim_sheers, &mut core.brushes],
            |time, sheer, brush| {
                solve_sheer(*time, sheer, brush);
            }
        );

        query!(
            [&mut anim.anim_sprites, &mut core.brushes],
            |sprite, brush| {
                solve_sprite(sprite, brush);
            }
        );
    }
}
