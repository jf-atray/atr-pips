use crate::tables::{CanvasId, MaterialId};

#[derive(Clone)]
pub struct Brush {
    pub canvas: CanvasId,
    pub material: MaterialId,
}