#[macro_use]
mod macros;

mod addition;
mod domain;
mod traits;
mod typed_map;
mod view;

#[allow(unused_imports)]
pub use {
    addition::Addition,
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