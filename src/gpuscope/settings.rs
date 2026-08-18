use wgpu::{Backends, PowerPreference, PresentMode};

/// Immutable settings used to create or recreate a [`Gpu`].
#[derive(Clone, Debug)]
pub struct GpuSettings {
    pub backends: Backends,
    pub power_preference: PowerPreference,
    pub present_mode: PresentMode,
    pub sample_count: u32,
    pub depth_enabled: bool,
}

impl Default for GpuSettings {
    fn default() -> Self {
        Self {
            backends: Backends::all(),
            power_preference: PowerPreference::HighPerformance,
            present_mode: PresentMode::AutoVsync,
            sample_count: 4,
            depth_enabled: true,
        }
    }
}
