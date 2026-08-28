use std::collections::HashMap;

use crate::assets::SpriteEntry;
use crate::gamescope::scene::{NoopScene, Scene, SceneAction, SceneContext};
use crate::gpuscope::Gpu;
use crate::input::Input;
use crate::spacial::camera::Camera;
use crate::addition::ExampleDomain;

#[derive(Debug)]
pub struct Game {
    pub domain: ExampleDomain,
    pub camera: Camera,
    pub asset_registry: HashMap<String, SpriteEntry>,
    pub input: Input,
    pub scene: Box<dyn Scene>,
}

impl Game {
    pub fn new(asset_registry: HashMap<String, SpriteEntry>) -> Self {
        Self {
            domain: ExampleDomain::default(),
            camera: Camera::new(),
            asset_registry,
            input: Input::new(),
            scene: Box::new(NoopScene),
        }
    }

    pub fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.scene = scene;
    }

    pub fn update(&mut self, dt: f32, aspect: f32, gpu: &mut Gpu) {
        let mut game_action = SceneAction::new();
        let mut ctx = SceneContext {
            dt,
            aspect,
            domain: &mut self.domain,
            asset_registry: &mut self.asset_registry,
            input: &mut self.input,
            camera: &mut self.camera,
            gpu,
            game_action: &mut game_action,
        };
        self.scene.update(&mut ctx);

        self.domain.update_solvers(dt, &self.input, &self.asset_registry);

        if let Some(next) = game_action.next_scene.take() {
            self.set_scene(next);
        }

        self.camera.update(aspect);
        self.input.end_frame();
    }
}
