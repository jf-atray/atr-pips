use std::{rc::Rc, sync::Arc};

use wgpu::{
    Device, Extent3d, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages,
};


pub struct RenderTargets {
    //virus
    sample_count: u32,
    depth_enabled: bool,
    //symptoms
    pub(crate) depth: Option<Rc<Texture>>,
    pub(crate) msaa_color: Option<Rc<Texture>>,
}

impl RenderTargets {
    pub fn make(
        device: &Device,
        surface_cfg: &SurfaceConfiguration,
        sample_count: u32,
        depth_enabled: bool,
    ) -> Self {
        let mut targets = Self {
            sample_count,
            depth_enabled,
            depth: None,
            msaa_color: None,
        };
        targets.rebuild(device, surface_cfg);
        targets
    }

    pub fn rebuild(&mut self, device: &Device, surface_cfg: &SurfaceConfiguration) {
        self.depth = self
            .depth_enabled
            .then(|| Rc::new(create_depth_texture(device, surface_cfg, self.sample_count)));

        self.msaa_color = (self.sample_count > 1)
            .then(|| Rc::new(create_msaa_texture(device, surface_cfg, self.sample_count)));
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn depth_enabled(&self) -> bool {
        self.depth_enabled
    }
}

fn create_msaa_texture(device: &Device, cfg: &SurfaceConfiguration, sample_count: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("msaa_color"),
        size: Extent3d {
            width: cfg.width,
            height: cfg.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: TextureDimension::D2,
        format: cfg.format,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

fn create_depth_texture(device: &Device, cfg: &SurfaceConfiguration, sample_count: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("depth"),
        size: Extent3d {
            width: cfg.width,
            height: cfg.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}
