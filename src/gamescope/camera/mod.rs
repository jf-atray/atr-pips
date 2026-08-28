use glam::Vec2;

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
