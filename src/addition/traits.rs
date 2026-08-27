use downcast_rs::{Downcast, impl_downcast};

use crate::ecs::partition::Partition;
use super::typed_map::TypedMap;

pub trait Tables: Downcast + Partition {}
pub trait Solver: Downcast {}

pub trait Solvers: Downcast {
    fn update(
        &mut self,
        dt: f32,
        tables: &mut TypedMap<dyn Tables>,
        scripts: &mut TypedMap<dyn Scripts>,
        signals: &mut TypedMap<dyn Signals>,
    );
}

pub trait Scripts: Downcast {}
pub trait Signals: Downcast {}

impl_downcast!(Tables);
impl_downcast!(Solvers);
impl_downcast!(Scripts);
impl_downcast!(Signals);
