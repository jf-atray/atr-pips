use std::{
    any::{Any, TypeId, try_as_dyn_mut}, collections::{HashMap, HashSet},
};

struct ExampleDomain {
    concept: HashSet<TypeId>,
    pub tables: HashMap<TypeId, Box<dyn Tables+'static>>,
    pub solvers: HashMap<TypeId, Box<dyn Solvers+'static>>,
    pub scripts: HashMap<TypeId, Box<dyn Scripts+'static>>,
    pub signals: HashMap<TypeId, Box<dyn Signals+'static>>,
}
impl ExampleDomain {
    pub fn get<T: Addition+'static>(&mut self) -> Option<AsViewMut<'_, T>> {
        let id = TypeId::of::<T>();
        let tables = self.tables.get_mut(&id) ?;
        let solvers = self.solvers.get_mut(&id) ?;
        let scripts = self.scripts.get_mut(&id) ?;
        let signals = self.signals.get_mut(&id) ?;

        let tables = tables.as_mut();
        let tables_any = tables as &mut dyn Any;
        let tables = tables_any.downcast_mut::<T::Tables>() ?;

        let solvers = solvers.as_mut();
        let tables_any = solvers as &mut dyn Any;
        let solvers = tables_any.downcast_mut::<T::Solvers>() ?;

        let scripts = scripts.as_mut();
        let tables_any = scripts as &mut dyn Any;
        let scripts = tables_any.downcast_mut::<T::Scripts>() ?;

        let signals = signals.as_mut();
        let tables_any = signals as &mut dyn Any;
        let signals = tables_any.downcast_mut::<T::Signals>() ?;

        let view = AsViewMut::<T>::new(tables, solvers, scripts, signals);
        Some(view)
    }
    pub fn add<T: Addition + 'static>(&mut self) -> Result<AsViewMut<'_,T>, ()> {
        let id = TypeId::of::<T>();

        //todo, make this a nice trait call
        let tables = T::make_tables();
        let boxed = Box::new(tables);
        let tables = boxed as Box<dyn Tables>;
        self.tables.insert(id, tables);

        let solvers = T::make_solvers();
        let boxed = Box::new(solvers);
        let solvers = boxed as Box<dyn Solvers>;
        self.solvers.insert(id, solvers);

        let scripts = T::make_scripts();
        let boxed = Box::new(scripts);
        let scripts = boxed as Box<dyn Scripts>;
        self.scripts.insert(id, scripts);

        let signals = T::make_signals();
        let boxed = Box::new(signals);
        let signals = boxed as Box<dyn Signals>;
        self.signals.insert(id, signals);

        //todo, make a helper impl over option and result
        self.get::<T>().ok_or_else(|| {
            if cfg!(not(debug_assertions)) {
                panic!()
            }
        })
    }
}

#[derive(Debug)]
struct ViewMut<'domain, T, K, M, N>
where
    T: Tables,
    K: Solvers,
    M: Scripts,
    N: Signals,
{
    pub tables: &'domain mut T,
    pub solvers: &'domain mut K,
    pub scripts: &'domain mut M,
    pub signals: &'domain mut N,
}
impl<'domain, T, K, M, N> ViewMut<'domain, T, K, M, N> 
where
    T: Tables,
    K: Solvers,
    M: Scripts,
    N: Signals ,
    {
    pub fn new(
        tables: &'domain mut T,
        solvers: &'domain mut K,
        scripts: &'domain mut M,
        signals: &'domain mut N,) -> Self {
        Self {
            tables,
            solvers,
            scripts,
            signals
        }

    }
}
trait Tables: Any {}
trait Solvers: Any {}
trait Solver: Any {}
impl<T: Solver> Solvers for T {}

trait Scripts: Any {}
trait Signals: Any {}
type AsViewMut<'a, A>
        = ViewMut<'a, <A as Addition>::Tables, <A as Addition>::Solvers, <A as Addition>::Scripts, <A as Addition>::Signals>;

trait Addition {
    //could always consume self to protect this once impl
    type Tables: Tables;
    type Solvers: Solvers;
    type Scripts: Scripts;
    type Signals: Signals;
    fn make_tables() -> Self::Tables;
    fn make_solvers() -> Self::Solvers;
    fn make_scripts() -> Self::Scripts;
    fn make_signals() -> Self::Signals;
}

//#[cfg(test)]
mod test {
    use std::assert_matches;

    use crate::addition::*;

    #[derive(Debug)]
    struct CowboyWorld {}
    #[derive(Debug)]
    struct CowboyTables {}
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
            CowboyTables {}
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
        let mut domain = ExampleDomain {
            concept: HashSet::new(),
            tables: HashMap::new(),
            solvers: HashMap::new(),
            scripts: HashMap::new(),
            signals: HashMap::new(),
        };

        let rslt = domain.add::<CowboyWorld>();
        println!("{rslt:#?}");
        assert_matches!(rslt, Ok(_));
    }
}
