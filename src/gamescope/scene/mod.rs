use std::cell::OnceCell;

use crate::assets::AssetRegistry;
use crate::gpuscope::Gpu;
use crate::input::Input;
use crate::scripting::scripts::Scripts;
use crate::scripting::solvers::Solvers;
use crate::spacial::camera::Camera;
use crate::tables::domain::Domain;

//todo, for more complex cases this needs an view struct with some mut some ref borrows
#[derive(Default)]
pub struct SceneAction {
    pub next_scene: OnceCell<Box<dyn Scene>>,
}

impl SceneAction {
    pub fn new() -> Self {
        Self {
            next_scene: OnceCell::new(),
        }
    }
}

pub struct SceneContext<'a> {
    pub dt: f32,
    pub aspect: f32,
    pub domain: &'a mut Domain,
    pub asset_registry: &'a mut AssetRegistry,
    pub input: &'a mut Input,
    pub scripts: &'a mut Scripts,
    pub solvers: &'a mut Solvers,
    pub camera: &'a mut Camera,
    pub gpu: &'a mut Gpu,
    pub game_action: &'a SceneAction,
}

pub trait Scene {
    fn update(&mut self, ctx: &mut SceneContext);
}

pub struct NoopScene;

impl Scene for NoopScene {
    fn update(&mut self, _ctx: &mut SceneContext) {}
}
