use crate::assets::AssetRegistry;
use crate::scripting::{Scripts, Solvers};
use crate::spacial::camera::Camera;
use crate::tables::domain::Domain;

pub struct Game {
    pub domain: Domain,
    pub camera: Camera,
    pub asset_registry: AssetRegistry,
    pub scripts: Scripts,
    pub solvers: Solvers,
}

impl Game {
    pub fn new(asset_registry: AssetRegistry) -> Self {
        Self {
            domain: Domain::new(),
            camera: Camera::new(),
            asset_registry,
            scripts: Scripts::new(),
            solvers: Solvers::new(),
        }
    }


    pub fn update(&mut self, dt: f32, aspect: f32) {
        self.scripts.update_enabled(dt, &mut self.domain, &self.solvers, &self.asset_registry);
        self.solvers.update_enabled(dt, &mut self.domain, &self.scripts, &self.asset_registry);
        
        self.camera.update(aspect);
        
    }
}
