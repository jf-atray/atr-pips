use glam::{Vec2, Vec3, Vec4};

use crate::assets::SpriteEntry;
use crate::ecs::{CanvasId, MaterialId};

#[derive(Clone, Debug)]
pub struct Brush {
    pub canvas: CanvasId,
    pub material: MaterialId,
    pub scale: Vec3,
    pub offset: Vec3,
    pub sheer: Vec2,
    pub color: Vec4,
}

impl Brush {
    pub fn new(canvas: CanvasId, material: MaterialId) -> Self {
        Self {
            canvas,
            material,
            scale: Vec3::ONE,
            offset: Vec3::ZERO,
            sheer: Vec2::ZERO,
            color: Vec4::ONE,
        }
    }

    pub fn from_sprite(sprite: &SpriteEntry) -> Self {
        Self {
            canvas: sprite.canvas,
            material: sprite.material,
            scale: Vec3::ONE,
            offset: Vec3::ZERO,
            sheer: Vec2::ZERO,
            color: Vec4::ONE,
        }
    }
}
