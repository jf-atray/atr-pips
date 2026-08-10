use wgpu::{
    Adapter, Backend, Backends, CommandEncoderDescriptor, CompositeAlphaMode,
    CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures, Features, Limits,
    MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    SurfaceColorSpace, TextureUsages, Trace,
};

use crate::libscope::Lib;

pub mod canvasing;
pub mod frame;
pub mod memory;
pub mod settings;
pub mod texture_cache;

mod device;
mod surface;
mod targets;

pub use device::DeviceContext;
pub use frame::Frame;
pub use settings::GpuSettings;
pub use surface::SurfaceScope;
pub use targets::RenderTargets;

pub use self::canvasing::CanvasRenderer;
pub use self::texture_cache::TextureScope;


pub struct GpuReady(pub GpuParts);

pub struct GpuParts {
    pub(crate) surface: Surface<'static>,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) cfg: SurfaceConfiguration,
}

impl GpuParts {
    pub async fn make(
        lib: &Lib,
        surface: Surface<'static>,
        width: u32,
        height: u32,
        settings: &GpuSettings,
    ) -> Self {
        let instance = &lib.inst;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: settings.power_preference,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .expect("no compatible adapter");

        let required_limits = match adapter.get_info().backend {
            Backend::Gl => Limits::downlevel_webgl2_defaults(),
            _ => Limits::defaults(),
        };

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("gpu"),
                required_features: Features::empty(),
                required_limits,
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await
            .expect("failed to request device");

        let surface_format = pick_surface_format(&surface, &adapter);
        let cfg = SurfaceConfiguration {
            width,
            height,
            present_mode: settings.present_mode,
            format: surface_format,
            color_space: SurfaceColorSpace::Auto,
            usage: TextureUsages::RENDER_ATTACHMENT,
            alpha_mode: CompositeAlphaMode::Auto,
            desired_maximum_frame_latency: 2,
            view_formats: vec![surface_format.add_srgb_suffix()],
        };

        Self {
            surface,
            adapter,
            device,
            queue,
            cfg,
        }
    }
}

pub struct Gpu {
    pub(crate) surface: SurfaceScope,
    pub(crate) device: DeviceContext,
    pub(crate) targets: RenderTargets,
}

impl Gpu {
    pub fn make(parts: GpuParts, settings: &GpuSettings) -> Self {
        let GpuParts {
            surface,
            adapter,
            device,
            queue,
            cfg,
        } = parts;

        surface.configure(&device, &cfg);

        let surface_scope = SurfaceScope { surface, cfg };

        let canvas_renderer = CanvasRenderer::make(device.clone());
        let texture_scope = TextureScope::new();

        let device_context = DeviceContext {
            adapter,
            device,
            queue,
            texture_scope,
            canvas_renderer,
        };

        let targets = RenderTargets::make(
            &device_context.device,
            &surface_scope.cfg,
            settings.sample_count,
            settings.depth_enabled,
        );

        Self {
            surface: surface_scope,
            device: device_context,
            targets,
        }
    }

    pub fn reconfigure(&mut self, width: u32, height: u32) {
        let device = &self.device.device;
        self.surface.reconfigure(device, width, height);
        self.targets
            .rebuild(device, &self.surface.cfg);
    }

    pub fn begin_frame(&mut self) -> Option<Frame> {
        match self.surface.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(st) | CurrentSurfaceTexture::Suboptimal(st) => {
                let encoder = self
                    .device
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor { label: Some("frame") });

                Some(Frame::new(st, encoder, &self.targets))
            }
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                let (w, h) = (self.surface.cfg.width, self.surface.cfg.height);
                self.reconfigure(w, h);
                None
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => None,
            CurrentSurfaceTexture::Validation => {
                log::error!("Surface::get_current_texture validation error");
                None
            }
        }
    }

    pub fn queue(&self) -> &Queue {
        &self.device.queue
    }

    pub fn surface_cfg(&self) -> &SurfaceConfiguration {
        &self.surface.cfg
    }

    pub fn settings(&self) -> GpuSettings {
        GpuSettings {
            backends: Backends::all(),
            power_preference: PowerPreference::HighPerformance,
            present_mode: self.surface.cfg.present_mode,
            sample_count: self.targets.sample_count(),
            depth_enabled: self.targets.depth_enabled(),
        }
    }
}

fn pick_surface_format(surface: &Surface, adapter: &Adapter) -> wgpu::TextureFormat {
    let capabilities = surface.get_capabilities(adapter);

    let best = capabilities
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .or_else(|| capabilities.formats.iter().find(|f| f.has_color_aspect()))
        .copied()
        .expect("no usable surface format");

    log::info!("surface format: {best:?}");
    best
}
