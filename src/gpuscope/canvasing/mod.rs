use wgpu::{Device, Queue, RenderPass};

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
