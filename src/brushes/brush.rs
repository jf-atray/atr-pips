use glam::{Vec2, Vec4};

use crate::tables::{CanvasId, MaterialId};

#[derive(Clone)]
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
}