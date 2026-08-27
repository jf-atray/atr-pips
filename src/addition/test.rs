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
        _dt: f32,
        tables: &mut TypedMap<dyn Tables>,
        _scripts: &mut TypedMap<dyn Scripts>,
        _signals: &mut TypedMap<dyn Signals>,
    ) {
        self.drawn += 1;
        let _ = CowboyWorld::tables(tables);
    }
}

#[derive(Debug, Default)]
struct BootSolver {
    worn: u32,
}

impl BootSolver {
    fn update(
        &mut self,
        _dt: f32,
        tables: &mut TypedMap<dyn Tables>,
        _scripts: &mut TypedMap<dyn Scripts>,
        _signals: &mut TypedMap<dyn Signals>,
    ) {
        self.worn += 1;
        let _ = CowboyWorld::tables(tables);
    }
}

addition! {
    #[derive(Debug)]
    CowboyWorld {
        tables: CowboyTables {
            hats: Class<u32, ()> = Class::new(GrowthStrategy::quart_kib::<u32>()),
        },
        solvers: CowboySolvers {
            hats: HatSolver = HatSolver::default(),
            boots: BootSolver = BootSolver::default(),
        },
        scripts: CowboyScripts {},
        signals: CowboySignals {},
    }
}
#[test]
pub fn creation() {
    let mut domain = ExampleDomain::default();
    let rslt = domain.add::<CowboyWorld>();
    println!("{rslt:#?}");
    assert_matches!(rslt, Ok(_));

    let cowboy_tables = CowboyWorld::tables(&mut domain.tables)
        .expect("Expect cowboyworld to exist by now.");
    let _ = cowboy_tables;
}
