use glam::{Quat, Vec2, Vec3, Vec4};


use crate::brushes::Brush;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::ecs::scope::{Maker, Scope};
use crate::ecs::{CanvasId, MaterialId};
use crate::you_first::gamejam::roller::components::{
    BrushFlip, RollerDepth, RollerPlayer, RollerView,
};
use crate::you_first::gamejam::roller::projection::WALK_SPEED;

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
        scope.core.with(
            Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush,
            name,
            Motion::default(),
        );
        let rv = scope.view::<RollerView>().unwrap();
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
    move |scope: &mut Scope| {
        body.make_into(scope);
        let rv = scope.view::<RollerView>().unwrap();
        rv.roller_players = Some(roller_player);
    }
}
