use glam::Vec4;
use glam::prelude::Vec3;

use crate::assets::SpriteEntry;
use crate::ecs::{CanvasId, MaterialId};

#[derive(Clone, Debug)]
pub struct Brush {
    pub canvas: CanvasId,
    pub material: MaterialId,
    pub scale: Vec3,
    pub color: Vec4,
}

impl Brush {
    pub fn new(canvas: CanvasId, material: MaterialId) -> Self {
        Self {
            canvas,
            material,
            scale: Vec3::ONE,
            color: Vec4::ONE,
        }
    }

    pub fn from_sprite(sprite: &SpriteEntry) -> Self {
        Self {
            canvas: sprite.canvas,
            material: sprite.material,
            scale: Vec3::ONE,
            color: Vec4::ONE,
        }
    }
}
