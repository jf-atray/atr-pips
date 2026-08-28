use std::collections::HashMap;

use downcast_rs::{Downcast, impl_downcast};

use crate::assets::SpriteEntry;
use crate::ecs::partition::Partition;
use crate::input::Input;

use super::domain::{Pips, ScriptsMap, SignalsMap};

pub trait Tables: Downcast + Partition + std::fmt::Debug {}
pub trait Solver: Downcast + std::fmt::Debug {}

pub trait Solvers: Downcast + std::fmt::Debug {
    fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        input: &Input,
        asset_registry: &HashMap<String, SpriteEntry>,
    );
}

pub trait Scripts: Downcast + std::fmt::Debug {}
pub trait Signals: Downcast + std::fmt::Debug {}

impl_downcast!(Tables);
impl_downcast!(Solvers);
impl_downcast!(Scripts);
impl_downcast!(Signals);
