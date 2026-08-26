use std::assert_matches;

use crate::{
    addition::*,
    ecs::{class::Class, class_strategy::GrowthStrategy},
};

#[derive(Debug)]
struct CowboyWorld {}
#[derive(Debug)]
struct CowboyTables {
    hats: Class<u32, ()>,
}
impl Tables for CowboyTables {}
#[derive(Debug)]
struct CowboySolver {}
impl Solver for CowboySolver {}
impl Scripts for () {}
impl Signals for () {}
impl Addition for CowboyWorld {
    type Tables = CowboyTables;
    type Solvers = CowboySolver;
    type Scripts = ();
    type Signals = ();

    fn make_tables() -> Self::Tables {
        CowboyTables {
            hats: Class::new(GrowthStrategy::quart_kib::<u32>()),
        }
    }

    fn make_solvers() -> Self::Solvers {
        CowboySolver {}
    }

    fn make_scripts() -> Self::Scripts {
        ()
    }

    fn make_signals() -> Self::Signals {
        ()
    }
}
#[test]
pub fn creation() {
    let mut domain = ExampleDomain::default();

    let rslt = domain.add::<CowboyWorld>();
    println!("{rslt:#?}");
    assert_matches!(rslt, Ok(_));

    let cowboy_tables = domain
        .tables
        .get_mut::<CowboyWorld, CowboyTables>()
        .expect("Expect cowboyworld to exist by now.");
    let _ = cowboy_tables;
}
