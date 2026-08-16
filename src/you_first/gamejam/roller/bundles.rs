use glam::{Quat, Vec2, Vec3, Vec4, vec2, vec3};

use crate::bundle;
use crate::gamejam::roller::components::{DuelOpponent, RollerDepth, RollerPlayer};
use crate::gamejam::roller::projection::FAR_Z;
use crate::gpuscope::{CanvasId, canvas::MaterialId};
use crate::anim::{AnimKeyframe, AnimScale, AnimSheer, AnimSpin, AnimTime, AnimXyz};
use crate::gamejam::duel::state::{LivingAnimLib, TumbleweedAnimLib};
use crate::pip::{Transform, brush::{Brush, BrushFlip}, sprite_rect::SpriteRect};
use crate::scenes::SceneResources;

fn u8_to_f32_color(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

bundle! {
    pub struct RollerSpriteBundle {
        transform: Transform,
        brush: Brush,
        sprite_rect: SpriteRect,
        roller_depth: RollerDepth,
        brush_flip: BrushFlip,
    }
}

impl RollerSpriteBundle {
    pub fn new(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        canvas: CanvasId,
        mat: MaterialId,
    ) -> Self {
        let color = u8_to_f32_color(color);
        Self {
            transform: Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush: Brush {
                scale: vec3(0.0, 0.0, 1.0),
                color: Vec4::new(color[0], color[1], color[2], color[3]),
                ..Brush::new(canvas, mat)
            },
            sprite_rect: SpriteRect::full(base_size.x, base_size.y),
            roller_depth: RollerDepth { d, lateral, speed: 0.0, scalar: 1.0, lateral_speed: 0.0 },
            brush_flip: BrushFlip::default(),
        }
    }
    pub fn from_asset(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        material: &str,
        resources: &SceneResources<'_>,
    ) -> Self {
        let (canvas, mat) = resources.assets.get(material);
        Self::new(lateral, d, base_size, color, canvas, mat)
    }
}

bundle! {
    pub struct LivingRollerBundle {
        transform: Transform,
        brush: Brush,
        sprite_rect: SpriteRect,
        roller_depth: RollerDepth,
        brush_flip: BrushFlip,
        anim_keyframe: AnimKeyframe,
        anim_time: AnimTime,
        anim_scale: AnimScale,
        anim_sheer: AnimSheer,
        anim_xyz: AnimXyz,
    }
}

impl LivingRollerBundle {
    pub fn new(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        canvas: CanvasId,
        mat: MaterialId,
        living_anim: LivingAnimLib,
    ) -> Self {
        let color = u8_to_f32_color(color);
        Self {
            transform: Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush: Brush {
                scale: vec3(0.0, 0.0, 1.0),
                color: Vec4::new(color[0], color[1], color[2], color[3]),
                ..Brush::new(canvas, mat)
            },
            sprite_rect: SpriteRect::full(base_size.x, base_size.y),
            roller_depth: RollerDepth { d, lateral, speed: 0.0, scalar: 1.0, lateral_speed: 0.0 },
            brush_flip: BrushFlip::default(),
            anim_keyframe: AnimKeyframe {
                id: living_anim.root_anim,
                lib: living_anim.lib,
            },
            anim_time: AnimTime(f32::NAN),
            anim_scale: AnimScale::default(),
            anim_sheer: AnimSheer::default(),
            anim_xyz: AnimXyz::default(),
        }
    }

    pub fn from_asset(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        material: &str,
        living_anim: LivingAnimLib,
        resources: &SceneResources<'_>,
    ) -> Self {
        let (canvas, mat) = resources.assets.get(material);
        Self::new(lateral, d, base_size, color, canvas, mat, living_anim)
    }
}

bundle! {
    pub struct TumbleweedRollerBundle {
        transform: Transform,
        brush: Brush,
        sprite_rect: SpriteRect,
        roller_depth: RollerDepth,
        brush_flip: BrushFlip,
        anim_keyframe: AnimKeyframe,
        anim_time: AnimTime,
        anim_spin: AnimSpin,
        anim_xyz: AnimXyz,
    }
}

impl TumbleweedRollerBundle {
    pub fn new(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        canvas: CanvasId,
        mat: MaterialId,
        tumbleweed_anim: TumbleweedAnimLib,
    ) -> Self {
        let color = u8_to_f32_color(color);
        Self {
            transform: Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush: Brush {
                scale: vec3(0.0, 0.0, 1.0),
                color: Vec4::new(color[0], color[1], color[2], color[3]),
                ..Brush::new(canvas, mat)
            },
            sprite_rect: SpriteRect::full(base_size.x, base_size.y),
            roller_depth: RollerDepth { d, lateral, speed: 0.0, scalar: 1.0, lateral_speed: 0.0 },
            brush_flip: BrushFlip::default(),
            anim_keyframe: AnimKeyframe {
                id: tumbleweed_anim.root_anim,
                lib: tumbleweed_anim.lib,
            },
            anim_time: AnimTime(f32::NAN),
            anim_spin: AnimSpin::default(),
            anim_xyz: AnimXyz::default(),
        }
    }

    pub fn from_asset(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        material: &str,
        tumbleweed_anim: TumbleweedAnimLib,
        resources: &SceneResources<'_>,
    ) -> Self {
        let (canvas, mat) = resources.assets.get(material);
        Self::new(lateral, d, base_size, color, canvas, mat, tumbleweed_anim)
    }
}

bundle! {
    pub struct RollerOpponentBundle {
        transform: Transform,
        brush: Brush,
        sprite_rect: SpriteRect,
        roller_depth: RollerDepth,
        duel_opponent: DuelOpponent,
    }
}

impl RollerOpponentBundle {
    pub fn new(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        canvas: CanvasId,
        mat: MaterialId,
    ) -> Self {
        let color = u8_to_f32_color(color);
        Self {
            transform: Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush: Brush {
                scale: vec3(0.0, 0.0, 1.0),
                color: Vec4::new(color[0], color[1], color[2], color[3]),
                ..Brush::new(canvas, mat)
            },
            sprite_rect: SpriteRect::full(base_size.x, base_size.y),
            roller_depth: RollerDepth { d, lateral, speed: 0.0, scalar: 1.0, lateral_speed: 0.0 },
            duel_opponent: DuelOpponent,
        }
    }

    pub fn from_asset(
        lateral: f32,
        d: f32,
        base_size: Vec2,
        color: [u8; 4],
        material: &str,
        resources: &SceneResources<'_>,
    ) -> Self {
        let (canvas, mat) = resources.assets.get(material);
        Self::new(lateral, d, base_size, color, canvas, mat)
    }
}

bundle! {
    pub struct PlayerRollerBundle {
        transform: Transform,
        brush: Brush,
        sprite_rect: SpriteRect,
        roller_depth: RollerDepth,
        roller_player: RollerPlayer,
        anim_keyframe: AnimKeyframe,
        anim_time: AnimTime,
        anim_scale: AnimScale,
        anim_sheer: AnimSheer,
        anim_xyz: AnimXyz,
    }
}

impl PlayerRollerBundle {
    pub fn new(
        base_size: Vec2,
        color: [u8; 4],
        canvas: CanvasId,
        mat: MaterialId,
    ) -> Self {
        let color = u8_to_f32_color(color);
        Self {
            transform: Transform {
                xyz: vec2(0.0, 0.0).extend(0.0),
                rot: Quat::IDENTITY,
            },
            brush: Brush {
                scale: vec3(base_size.x, base_size.y, 1.0),
                color: Vec4::new(color[0], color[1], color[2], color[3]),
                ..Brush::new(canvas, mat)
            },
            sprite_rect: SpriteRect::full(base_size.x, base_size.y),
            roller_depth: RollerDepth { d: 2.0, lateral: 0.0, speed: 0.0, scalar: 1.0, lateral_speed: 0.0 },
            roller_player: RollerPlayer {
                walk_distance: 0.0,
                lateral: 0.0,
            },
            anim_keyframe: AnimKeyframe {
                id: crate::anim::AnimId::default(),
                lib: crate::anim::AnimLibId::default(),
            },
            anim_time: AnimTime(f32::NAN),
            anim_scale: AnimScale::default(),
            anim_sheer: AnimSheer::default(),
            anim_xyz: AnimXyz::default(),
        }
    }

    pub fn from_asset(
        base_size: Vec2,
        color: [u8; 4],
        material: &str,
        resources: &SceneResources<'_>,
    ) -> Self {
        let (canvas, mat) = resources.assets.get(material);
        Self::new(base_size, color, canvas, mat)
    }
}
