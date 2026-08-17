use glam::{Quat, Vec3, Vec4};

use crate::assets::AssetRegistry;
use crate::brushes::Brush;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::{CanvasId, MaterialId};
use crate::tables::scope::{Scope, Maker};
use crate::you_first::gamejam::roller::components::{BrushFlip, RollerDepth, RollerPlayer, RollerView};
use crate::you_first::gamejam::roller::projection::WALK_SPEED;

fn u8_to_f32_color(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

fn assemble_roller_sprite(
    scope: &mut Scope,
    canvas: CanvasId,
    mat: MaterialId,
    color: [f32; 4],
    roller_depth: RollerDepth,
    brush_flip: BrushFlip,
) {
    let mut brush = Brush::new(canvas, mat);
    brush.scale = Vec3::ONE;
    brush.color = Vec4::new(color[0], color[1], color[2], color[3]);
    scope.core.with(
        Transform {
            xyz: Vec3::ZERO,
            rot: Quat::IDENTITY,
        },
        brush,
        String::new(),
        Motion::default(),
    );
    let rv = scope.view::<RollerView>().unwrap();
    rv.roller_depths = Some(roller_depth);
    rv.brush_flips = Some(brush_flip);
}

pub fn roller_sprite_bundle(
    canvas: CanvasId,
    mat: MaterialId,
    lateral: f32,
    d: f32,
    color: [u8; 4],
    scalar: f32,
    lateral_speed: f32,
    is_flipped: bool,
) -> impl Maker {
    let color = u8_to_f32_color(color);
    move |scope: &mut Scope| {
        let roller_depth = RollerDepth {
            d,
            lateral,
            speed: 0.0,
            scalar,
            lateral_speed,
        };
        let brush_flip = BrushFlip { is_flipped };
        assemble_roller_sprite(scope, canvas, mat, color, roller_depth, brush_flip);
    }
}

pub fn roller_sprite_bundle_from_asset(
    material: &str,
    asset_registry: &AssetRegistry,
    lateral: f32,
    d: f32,
    color: [u8; 4],
    scalar: f32,
    lateral_speed: f32,
    is_flipped: bool,
) -> impl Maker {
    let sprite = asset_registry.get(material);
    let canvas = sprite.canvas;
    let mat = sprite.material;
    let color = u8_to_f32_color(color);
    move |scope: &mut Scope| {
        let roller_depth = RollerDepth {
            d,
            lateral,
            speed: 0.0,
            scalar,
            lateral_speed,
        };
        let brush_flip = BrushFlip { is_flipped };
        assemble_roller_sprite(scope, canvas, mat, color, roller_depth, brush_flip);
    }
}

pub fn player_roller_bundle(
    canvas: CanvasId,
    mat: MaterialId,
    lateral: f32,
    d: f32,
    color: [u8; 4],
    scalar: f32,
) -> impl Maker {
    let color = u8_to_f32_color(color);
    move |scope: &mut Scope| {
        let mut brush = Brush::new(canvas, mat);
        brush.scale = Vec3::ONE;
        brush.color = Vec4::new(color[0], color[1], color[2], color[3]);
        scope.core.with(
            Transform {
                xyz: Vec3::ZERO,
                rot: Quat::IDENTITY,
            },
            brush,
            "player".to_string(),
            Motion::default(),
        );
        let roller_depth = RollerDepth {
            d,
            lateral,
            speed: -WALK_SPEED,
            scalar,
            lateral_speed: 0.0,
        };
        let roller_player = RollerPlayer {
            walk_distance: 0.0,
            lateral,
        };
        let rv = scope.view::<RollerView>().unwrap();
        rv.with(roller_depth, roller_player, BrushFlip { is_flipped: false });
    }
}
