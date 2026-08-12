use crate::assets::AssetRegistry;
use crate::scripting::{Scripts, Solvers};
use crate::tables::tables::Tables;
use crate::tables::domain::Domain;
use crate::tables::PipId;

pub trait Scene {
    fn name(&self) -> &str;
    fn player(&self) -> Option<PipId>;
    fn register_tables(&self, tables: &mut Tables);
    fn unregister_tables(&self, tables: &mut Tables);
    fn populate(&mut self, registry: &AssetRegistry, domain: &mut Domain);
    fn setup(&mut self, scripts: &mut Scripts, solvers: &mut Solvers);
    fn teardown(&self, scripts: &mut Scripts, solvers: &mut Solvers);
    fn is_complete(&mut self, dt: f32, domain: &Domain) -> bool;
}

pub struct SceneAccess {
    pub current: Box<dyn Scene>,
    pub order: Vec<Box<dyn Fn() -> Box<dyn Scene>>>,
    pub index: usize,
}

impl SceneAccess {
    pub fn next(&mut self) -> Box<dyn Scene> {
        assert!(!self.order.is_empty(), "SceneAccess has no scene factories");
        self.index = (self.index + 1) % self.order.len();
        self.order[self.index]()
    }
}
