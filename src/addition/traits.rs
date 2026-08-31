use std::collections::HashMap;

use downcast_rs::{Downcast, impl_downcast};

use crate::assets::SpriteEntry;
use crate::ecs::Table;
use crate::ecs::partition::Partition;
use crate::input::Input;

use super::domain::{Pips, ScriptsMap, SignalsMap};

pub trait Tables: Downcast + Partition + std::fmt::Debug {
    fn for_each_table(&self, f: &mut dyn FnMut(&'static str, &dyn Table));
}
pub trait Solver: Downcast + std::fmt::Debug {}

pub trait Solvers: Downcast + std::fmt::Debug {
    fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        input: &mut Input,
        asset_registry: &HashMap<String, SpriteEntry>,
    );

    fn for_each_solver(&mut self, f: &mut dyn FnMut(&'static str, &mut dyn Solver));
}

pub trait Scripts: Downcast + std::fmt::Debug {}
pub trait Signals: Downcast + std::fmt::Debug {}

impl_downcast!(Tables);
impl_downcast!(Solvers);
impl_downcast!(Scripts);
impl_downcast!(Signals);
