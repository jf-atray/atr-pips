use wgpu::{Adapter, Device, Queue};

use super::canvasing::CanvasRenderer;
use super::texture_cache::TextureCache;

//if we lose the gpu device, these are the things tha have to be rebuilt
pub struct DeviceContext {
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) texture_cache: TextureCache,

    //even you- pipelines belong to devices right?
    pub(crate) canvas_renderer: CanvasRenderer,
}
