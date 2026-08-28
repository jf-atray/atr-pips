use std::cell::OnceCell;
use std::collections::HashMap;

use crate::assets::SpriteEntry;
use crate::gpuscope::Gpu;
use crate::input::Input;
use crate::spacial::camera::Camera;
use crate::addition::ExampleDomain;

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
    pub domain: &'a mut ExampleDomain,
    pub asset_registry: &'a mut HashMap<String, SpriteEntry>,
    pub input: &'a mut Input,
    pub camera: &'a mut Camera,
    pub gpu: &'a mut Gpu,
    pub game_action: &'a SceneAction,
}

pub trait Scene: std::fmt::Debug {
    fn update(&mut self, ctx: &mut SceneContext);
}

#[derive(Debug)]
pub struct NoopScene;

impl Scene for NoopScene {
    fn update(&mut self, _ctx: &mut SceneContext) {}
}
