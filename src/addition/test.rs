use std::assert_matches;

use crate::{
    addition::*,
    ecs::{class::Class, class_strategy::GrowthStrategy},
};

#[derive(Debug, Default)]
struct HatSolver {
    drawn: u32,
}

impl HatSolver {
    fn update(
        &mut self,
        tables: &mut TypedMap<dyn Tables>,
        _scripts: &mut TypedMap<dyn Scripts>,
        _signals: &mut TypedMap<dyn Signals>,
    ) {
        self.drawn += 1;
        let _ = tables.get_mut::<CowboyWorld, CowboyTables>();
    }
}

#[derive(Debug, Default)]
struct BootSolver {
    worn: u32,
}

impl BootSolver {
    fn update(
        &mut self,
        tables: &mut TypedMap<dyn Tables>,
        _scripts: &mut TypedMap<dyn Scripts>,
        _signals: &mut TypedMap<dyn Signals>,
    ) {
        self.worn += 1;
        let _ = tables.get_mut::<CowboyWorld, CowboyTables>();
    }
}

addition! {
    #[derive(Debug)]
    CowboyWorld {
        tables: CowboyTables {
            hats: Class<u32, ()>,
        } = CowboyTables {
            hats: Class::new(GrowthStrategy::quart_kib::<u32>()),
        },
        solvers: CowboySolvers {
            hats: HatSolver,
            boots: BootSolver,
        } = CowboySolvers {
            hats: HatSolver::default(),
            boots: BootSolver::default(),
        },
        scripts: CowboyScripts {} = CowboyScripts {},
        signals: CowboySignals {} = CowboySignals {},
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
