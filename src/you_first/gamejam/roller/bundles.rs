use glam::{Quat, Vec2, Vec3, Vec4};


use crate::brushes::Brush;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::ecs::scope::{Maker, Scope};
use crate::ecs::core::CoreWorld;
use crate::ecs::{CanvasId, MaterialId};
use crate::you_first::gamejam::roller::components::{
    BrushFlip, RollerDepth, RollerPlayer, RollerWorld,
};
use crate::you_first::gamejam::roller::projection::WALK_SPEED;
use crate::you_first::gamejam::duel::state::LivingAnimLib;
use crate::anims::{AnimKeyframe, AnimScale, AnimSheer, AnimTime, AnimWorld, AnimXyz};
use crate::assets::SpriteEntry;


pub(crate) fn roller_body(
    canvas: CanvasId,
    mat: MaterialId,
    brush_scale: Vec3,
    name: String,
    roller_depth: RollerDepth,
    brush_flip: BrushFlip,
) -> impl Maker {
    move |scope: &mut Scope| {
        let mut brush = Brush::new(canvas, mat);
        brush.scale = brush_scale;
        brush.color = Vec4::ONE;
        scope.view::<CoreWorld>().unwrap().with(
            Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush,
            name,
            Motion::default(),
        );
        let rv = scope.view::<RollerWorld>().unwrap();
        rv.roller_depths = Some(roller_depth);
        rv.brush_flips = Some(brush_flip);
    }
}

pub fn player_roller_bundle(
    canvas: CanvasId,
    mat: MaterialId,
    lateral: f32,
    d: f32,
    size: Vec2,
    natural_scale: Vec2,
    scalar: f32,
    living_anim: &LivingAnimLib,
) -> impl Maker {
    let base_scale = size / natural_scale;
    let roller_depth = RollerDepth {
        d,
        lateral,
        speed: -WALK_SPEED,
        scalar,
        lateral_speed: 0.0,
        base_scale,
    };
    let brush_flip = BrushFlip { is_flipped: false };
    let body = roller_body(
        canvas,
        mat,
        Vec3::new(base_scale.x, base_scale.y, 1.0),
        "player".to_string(),
        roller_depth,
        brush_flip,
    );
    let roller_player = RollerPlayer {
        walk_distance: 0.0,
        lateral,
    };
    let lib = living_anim.lib;
    let root = living_anim.root_anim;
    move |scope: &mut Scope| {
        body.make_into(scope);
        let rv = scope.view::<RollerWorld>().unwrap();
        rv.roller_players = Some(roller_player);

        let av = scope.view::<AnimWorld>().unwrap();
        av.anim_times = Some(AnimTime(0.0));
        av.anim_keyframes = Some(AnimKeyframe { id: root, lib });
        av.anim_scales = Some(AnimScale::default());
        av.anim_sheers = Some(AnimSheer::default());
        av.anim_xyzs = Some(AnimXyz::default());
    }
}

pub fn living_roller_bundle(
    lateral: f32,
    d: f32,
    base_size: Vec2,
    color: Vec4,
    sprite: &SpriteEntry,
    living_anim: &LivingAnimLib,
    name: impl Into<String>,
) -> impl Maker {
    let base_scale = base_size / sprite.natural_scale;
    let mut brush = Brush::new(sprite.canvas, sprite.material);
    brush.color = color;
    let name = name.into();

    let roller_depth = RollerDepth {
        d,
        lateral,
        speed: 0.0,
        scalar: 1.0,
        lateral_speed: 0.0,
        base_scale,
    };

    let lib = living_anim.lib;
    let root = living_anim.root_anim;

    move |scope: &mut Scope| {
        scope.view::<CoreWorld>().unwrap().with(
            Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush,
            name,
            Motion::default(),
        );

        let rv = scope.view::<RollerWorld>().unwrap();
        rv.roller_depths = Some(roller_depth);
        rv.brush_flips = Some(BrushFlip { is_flipped: false });

        let av = scope.view::<AnimWorld>().unwrap();
        av.anim_times = Some(AnimTime(0.0));
        av.anim_keyframes = Some(AnimKeyframe { id: root, lib });
        av.anim_scales = Some(AnimScale::default());
        av.anim_sheers = Some(AnimSheer::default());
        av.anim_xyzs = Some(AnimXyz::default());
    }
}
