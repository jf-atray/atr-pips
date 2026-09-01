#[macro_use]
mod macros;

mod core;
mod domain;
mod traits;
mod typed_map;
mod view;

#[allow(unused_imports)]
pub use {
    core::Addition,
    domain::{
        ExampleDomain,
        Pips,
        TablesMap,
        SolversMap,
        ScriptsMap,
        SignalsMap,
        Ids,
        AnimLibs,
    },
    traits::{Signals, Solver, Solvers, Tables, Scripts},
    typed_map::Polysystem,
};

#[allow(unused_imports)]
pub use crate::ecs::scope::Scope;