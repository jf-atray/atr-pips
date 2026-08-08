use crate::tables::{CanvasId, MaterialId};

pub struct Brush {
    pub canvas: CanvasId,
    pub material: MaterialId,
}