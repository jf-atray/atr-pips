use wgpu::{Device, Surface, SurfaceConfiguration};

//try to publicly lock the w/h to a surface configure call
//mby move to a dirty enum so it happens only once a frame
pub struct SurfaceScope {
    pub(crate) surface: Surface<'static>,
    pub(crate) cfg: SurfaceConfiguration,
}

impl SurfaceScope {
    pub fn reconfigure(&mut self, device: &Device, width: u32, height: u32) {
        self.cfg.width = width;
        self.cfg.height = height;
        self.surface.configure(device, &self.cfg);
    }

    pub(crate) fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub(crate) fn cfg(&self) -> &SurfaceConfiguration {
        &self.cfg
    }
}
