use std::collections::HashMap;

use bumpalo::Bump;
use winit::keyboard::KeyCode;
use winit::event::MouseButton;

use crate::assets::SpriteEntry;
use crate::gamescope::camera::{CameraDirector, CameraMode};
use crate::gamescope::scene::{NoopScene, Scene, SceneAction, SceneContext};
use crate::gpuscope::Gpu;
use crate::input::{AxisConfig, Input};
use crate::spacial::camera::Camera;
use crate::addition::ExampleDomain;

#[derive(Debug)]
pub struct Game {
    pub domain: ExampleDomain,
    pub camera: Camera,
    pub camera_director: CameraDirector,
    pub camera_mode: CameraMode,
    pub asset_registry: HashMap<String, SpriteEntry>,
    pub input: Input,
    pub scene: Box<dyn Scene>,
}

impl Game {
    pub fn new(asset_registry: HashMap<String, SpriteEntry>) -> Self {
        let camera = Camera::new();
        let mut input = Input::new();

        input.add_axis(
            "camera_pan_x",
            AxisConfig::new(vec![KeyCode::KeyD], vec![KeyCode::KeyA]),
        );
        input.add_axis(
            "camera_pan_y",
            AxisConfig::new(vec![KeyCode::KeyW], vec![KeyCode::KeyS]),
        );
        input.add_axis(
            "camera_zoom",
            AxisConfig::new(vec![KeyCode::KeyQ], vec![KeyCode::KeyE]),
        );
        input.add_axis(
            "click",
            AxisConfig::new(vec![], vec![]).with_mouse(vec![MouseButton::Left], vec![]),
        );

        Self {
            domain: ExampleDomain::default(),
            camera,
            camera_director: CameraDirector::new(camera.pos, camera.zoom, 20.0, 30.0),
            camera_mode: CameraMode {
                pan: crate::gamescope::camera::PanSource::Fixed { value: camera.pos },
                zoom: crate::gamescope::camera::ZoomSource::Fixed { value: camera.zoom },
            },
            asset_registry,
            input,
            scene: Box::new(NoopScene),
        }
    }

    pub fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.scene = scene;
    }

    pub fn update(&mut self, dt: f32, aspect: f32, gpu: &mut Gpu, _arena: &mut Bump) {
        let mut game_action = SceneAction::new();
        let mut ctx = SceneContext {
            dt,
            aspect,
            domain: &mut self.domain,
            asset_registry: &mut self.asset_registry,
            input: &mut self.input,
            camera: &mut self.camera,
            camera_mode: &mut self.camera_mode,
            gpu,
            game_action: &mut game_action,
        };
        self.scene.update(&mut ctx);

        let win = self.domain.signals.core.window_size;
        let mouse_px = self.input.mouse.pos;
        let ndc_x = (mouse_px.x / win.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (mouse_px.y / win.y) * 2.0;
        let (view_w, view_h) = if aspect >= 1.0 {
            (self.camera.zoom, self.camera.zoom / aspect)
        } else {
            (self.camera.zoom * aspect, self.camera.zoom)
        };
        self.domain.signals.core.mouse_world = glam::Vec2::new(
            self.camera.pos.x + ndc_x * view_w * 0.5,
            self.camera.pos.y + ndc_y * view_h * 0.5,
        );

        self.domain
            .update_solvers(dt, &mut self.input, &self.asset_registry);

        let target = self
            .camera_mode
            .target(&self.input, &self.domain.pips, &self.camera);
        self.camera_director.set_pan_target(target.pan);
        self.camera_director.set_zoom_target(target.zoom);
        self.camera_director.update(&mut self.camera, dt);

        if let Some(next) = game_action.next_scene.take() {
            self.set_scene(next);
        }

        self.camera.update(aspect);
        self.input.end_frame();
    }
}
