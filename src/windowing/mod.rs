use std::sync::Arc;

use winit::window::Window;

#[derive(Debug)]
pub struct Windowing {
    //winit aggressively needs this in an arc
    pub window: Arc<Window>,
    pub width: u32,
    pub height: u32,
}

impl Windowing {
    pub fn new(window: Window, width: u32, height: u32) -> Self {
        Self {
            window: Arc::new(window),
            width,
            height,
        }
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}
