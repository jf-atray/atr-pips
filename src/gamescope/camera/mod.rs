use glam::Vec2;

use crate::addition::Pips;
use crate::ecs::PipId;
use crate::input::Input;
use crate::seek::{solve_seek, Seek};
use crate::spacial::camera::Camera;

#[derive(Clone, Debug)]
pub struct CameraDirector {
    pub pan: Seek<Vec2>,
    pub zoom: Seek<f32>,
}

impl CameraDirector {
    pub fn new(initial_pos: Vec2, initial_zoom: f32, pan_speed: f32, zoom_speed: f32) -> Self {
        Self {
            pan: Seek::with_speed(initial_pos, pan_speed),
            zoom: Seek::with_speed(initial_zoom, zoom_speed),
        }
    }

    pub fn set_pan_target(&mut self, target: Vec2) {
        self.pan.goal = target;
    }

    pub fn set_zoom_target(&mut self, target: f32) {
        self.zoom.goal = target;
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        solve_seek(&mut camera.pos, &self.pan, dt);
        solve_seek(&mut camera.zoom, &self.zoom, dt);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraTarget {
    pub pan: Vec2,
    pub zoom: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum PanSource {
    Manual { look: f32 },
    Track { target: PipId },
    Fixed { value: Vec2 },
}

impl PanSource {
    pub fn resolve(&self, input: &Input, pips: &Pips, camera: &Camera) -> Vec2 {
        match *self {
            PanSource::Manual { look } => {
                let pan_input = Vec2::new(
                    input.axes.value("camera_pan_x"),
                    input.axes.value("camera_pan_y"),
                );
                camera.pos + pan_input * look
            }
            PanSource::Track { target } => pips
                .ids
                .get(target)
                .map(|ptr| &pips.tables.core.xforms[ptr])
                .map(|transform| transform.xyz.truncate())
                .unwrap_or(camera.pos),
            PanSource::Fixed { value } => value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ZoomSource {
    Manual { look: f32 },
    Fixed { value: f32 },
}

impl ZoomSource {
    pub fn resolve(&self, input: &Input, camera: &Camera) -> f32 {
        match *self {
            ZoomSource::Manual { look } => {
                camera.zoom + input.axes.value("camera_zoom") * look
            }
            ZoomSource::Fixed { value } => value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraMode {
    pub pan: PanSource,
    pub zoom: ZoomSource,
}

impl CameraMode {
    pub fn target(&self, input: &Input, pips: &Pips, camera: &Camera) -> CameraTarget {
        CameraTarget {
            pan: self.pan.resolve(input, pips, camera),
            zoom: self.zoom.resolve(input, camera),
        }
    }
}
