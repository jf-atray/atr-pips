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
    domain::ExampleDomain,
    traits::{Signals, Solver, Solvers, Tables, Scripts},
    typed_map::TypedMap,
};

#[cfg(test)]
mod test;
