
use std::collections::HashMap;

use crate::assets::SpriteEntry;
use crate::gamescope::scene::{NoopScene, Scene, SceneAction, SceneContext};
use crate::gpuscope::Gpu;
use crate::input::Input;
use crate::scripting::{Scripts, Solvers};
use crate::spacial::camera::Camera;
use crate::tables::domain::Domain;

pub struct Game {
    pub domain: Domain,
    pub camera: Camera,
    pub asset_registry: HashMap<String, SpriteEntry>,
    pub input: Input,
    pub scripts: Scripts,
    pub solvers: Solvers,
    pub scene: Box<dyn Scene>,
}

impl Game {
    pub fn new(asset_registry: HashMap<String, SpriteEntry>) -> Self {
        Self {
            domain: Domain::new(),
            camera: Camera::new(),
            asset_registry,
            input: Input::new(),
            scripts: Scripts::new(),
            solvers: Solvers::new(),
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
            scripts: &mut self.scripts,
            solvers: &mut self.solvers,
            camera: &mut self.camera,
            gpu,
            game_action: &mut game_action,
        };
        self.scene.update(&mut ctx);

        self.scripts.update_enabled(
            dt,
            &mut self.domain,
            &self.solvers,
            &self.asset_registry,
            &self.input,
            &game_action,
        );
        self.solvers.update_enabled(
            dt,
            &mut self.domain,
            &self.scripts,
            &self.asset_registry,
            &self.input,
            &game_action,
        );

        if let Some(next) = game_action.next_scene.take() {
            self.set_scene(next);
        }

        self.camera.update(aspect);
        self.input.end_frame();
    }
}
