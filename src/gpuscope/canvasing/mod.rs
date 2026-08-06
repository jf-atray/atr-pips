use slotmap::{SlotMap, SparseSecondaryMap, new_key_type};
use wgpu::{Device, Queue, RenderPass};

new_key_type! {
    //should we define classes with canvas ID since it can affect the kinds of queries we want to run?
    pub struct CanvasId;
    pub struct MaterialId;
}

//canvas ID probably just points to solid resources
//all the "instance download" and "calling things" logic _belongs_ to the solver
pub struct SimpleCanvasDesign {
    
}
pub struct SimpleCanvasSolver {
    canvases: SparseSecondaryMap<CanvasId, SimpleCanvasDesign>,
}
pub struct ComplexCanvasDesign {

}
pub struct ComplexCanvasSolver {
    canvases: SparseSecondaryMap<CanvasId, ComplexCanvasDesign>,
}
pub struct Canvasing {
    canvases: SlotMap<CanvasId, EveryCanvas>,
}
pub struct EveryCanvas {
    pub bind: wgpu::RenderPipeline,
    pub layout: wgpu::PipelineLayout,
}
impl EveryCanvas {
}
pub struct CanvasAssociation {
    every_canvas: EveryCanvas,
    dispatch: Box<dyn CanvasSolvable>,
}
pub trait CanvasSolvable {

}

//wip move in from You First
pub struct CanvasRenderer;

impl CanvasRenderer {
    pub fn new(_device: &Device, _queue: &Queue) -> Self {
        Self
    }
}
