use std::sync::Arc;

use wgpu::{Backends, Instance, InstanceDescriptor, InstanceFlags, Surface, SurfaceTarget};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::gpuscope::{GpuParts, GpuSettings};

pub struct Lib {
    pub inst: Instance,
}

impl Lib {
    pub fn new(hand: OwnedDisplayHandle) -> Self {
        let flags = if cfg!(debug_assertions) {
            InstanceFlags::debugging()
        } else {
            InstanceFlags::empty()
        };

        let inst = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags,
            ..InstanceDescriptor::new_with_display_handle(Box::new(hand))
        });

        Self { inst }
    }

    pub fn make_surface(&self, window: Arc<Window>) -> Surface<'static> {
        self.inst
            .create_surface(SurfaceTarget::from_window_without_display(window))
            .expect("surface is required as a render target")
    }

    pub async fn make_gpu_parts(
        &self,
        surface: Surface<'static>,
        width: u32,
        height: u32,
    ) -> GpuParts {
        let settings = &GpuSettings::default();
        GpuParts::make(self, surface, width, height, settings).await
    }
}
