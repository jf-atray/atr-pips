use crate::assets::AssetRegistry;
use crate::gamescope::scene::{NoopScene, Scene, SceneContext};
use crate::gpuscope::Gpu;
use crate::scripting::{Scripts, Solvers};
use crate::spacial::camera::Camera;
use crate::tables::domain::Domain;

pub struct Game {
    pub domain: Domain,
    pub camera: Camera,
    pub asset_registry: AssetRegistry,
    pub scripts: Scripts,
    pub solvers: Solvers,
    pub scene: Box<dyn Scene>,
}

impl Game {
    pub fn new(asset_registry: AssetRegistry) -> Self {
        Self {
            domain: Domain::new(),
            camera: Camera::new(),
            asset_registry,
            scripts: Scripts::new(),
            solvers: Solvers::new(),
            scene: Box::new(NoopScene),
        }
    }

    pub fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.scene = scene;
    }

    pub fn update(&mut self, dt: f32, aspect: f32, gpu: &mut Gpu) {
        let mut ctx = SceneContext {
            dt,
            aspect,
            domain: &mut self.domain,
            asset_registry: &mut self.asset_registry,
            scripts: &mut self.scripts,
            solvers: &mut self.solvers,
            camera: &mut self.camera,
            gpu,
        };
        self.scene.update(&mut ctx);

        self.scripts.update_enabled(dt, &mut self.domain, &self.solvers, &self.asset_registry);
        self.solvers.update_enabled(dt, &mut self.domain, &self.scripts, &self.asset_registry);

        self.camera.update(aspect);
    }
}
