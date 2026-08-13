use glam::{Vec2, Vec4};

use crate::assets::SpriteEntry;
use crate::tables::{CanvasId, MaterialId};

#[derive(Clone, Debug)]
pub struct Brush {
    pub canvas: CanvasId,
    pub material: MaterialId,
    pub scale: Vec2,
    pub color: Vec4,
}

impl Brush {
    pub fn new(canvas: CanvasId, material: MaterialId) -> Self {
        Self {
            canvas,
            material,
            scale: Vec2::ONE,
            color: Vec4::ONE,
        }
    }

    pub fn from_sprite(sprite: &SpriteEntry) -> Self {
        Self {
            canvas: sprite.canvas,
            material: sprite.material,
            scale: sprite.natural_scale,
            color: Vec4::ONE,
        }
    }
}