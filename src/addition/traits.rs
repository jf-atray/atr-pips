use downcast_rs::{Downcast, impl_downcast};

use crate::ecs::partition::Partition;
use super::domain::{TablesMap, SolversMap, ScriptsMap, SignalsMap};

pub trait Tables: Downcast + Partition {}
pub trait Solver: Downcast {}

pub trait Solvers: Downcast {
    fn update(
        &mut self,
        dt: f32,
        tables: &mut TablesMap,
        scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
    );
}

pub trait Scripts: Downcast {}
pub trait Signals: Downcast {}

impl_downcast!(Tables);
impl_downcast!(Solvers);
impl_downcast!(Scripts);
impl_downcast!(Signals);
