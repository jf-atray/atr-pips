use glam::Vec2;

use crate::ecs::{CanvasId, MaterialId};

#[derive(Clone, Debug)]
pub struct SpriteEntry {
    pub canvas: CanvasId,
    pub material: MaterialId,
    pub natural_scale: Vec2,
}