use std::{
    any::{Any, TypeId, try_as_dyn_mut}, collections::{HashMap, HashSet}, fmt::Debug,
};

#[derive(Default, Debug)]
struct ExampleDomain {
    pub tables: TypedMap<dyn Tables>,
    pub solvers: TypedMap<dyn Solvers>,
    pub scripts: TypedMap<dyn Scripts>,
    pub signals: TypedMap<dyn Signals>,
}

//todo I don't like this. Check nightly or some standard std impl
trait AsAny : Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
struct TypedMap<T: ?Sized>(HashMap<TypeId, Box<T>>);
impl<T: ?Sized> TypedMap<T> {
    pub fn insert<K: 'static>(&mut self, t: Box<T>) {
        let id = TypeId::of::<K>();
        self.0.insert(id, t);
    }
}
impl<T: ?Sized> TypedMap<T>
where T: Tables+AsAny {
    pub fn get_tables<K: Addition+'static>(&mut self) -> Option<&mut K::Tables> {
        let id = TypeId::of::<K>();
        let tables = self.0.get_mut(&id) ?;
        
        let tables = tables.as_mut();
        let tables_any = tables.as_any_mut();
        tables_any.downcast_mut::<K::Tables>()
    }
}
impl<T: ?Sized> TypedMap<T>
where T: Solvers+AsAny {
    pub fn get_solvers<K: Addition+'static>(&mut self) -> Option<&mut K::Solvers> {
        let id = TypeId::of::<K>();
        let tables = self.0.get_mut(&id) ?;
        
        let tables = tables.as_mut();
        let tables_any = tables.as_any_mut();
        tables_any.downcast_mut::<K::Solvers>()
    }
}
impl<T: ?Sized> TypedMap<T>
where T: Scripts+AsAny {
    pub fn get_scripts<K: Addition+'static>(&mut self) -> Option<&mut K::Scripts> {
        let id = TypeId::of::<K>();
        let tables = self.0.get_mut(&id) ?;
        
        let tables = tables.as_mut();
        let tables_any = tables.as_any_mut();
        tables_any.downcast_mut::<K::Scripts>()
    }
}
impl<T: ?Sized> TypedMap<T>
where T: Signals+AsAny {
    pub fn get_signals<K: Addition+'static>(&mut self) -> Option<&mut K::Signals> {
        let id = TypeId::of::<K>();
        let tables = self.0.get_mut(&id) ?;
        
        let tables = tables.as_mut();
        let tables_any = tables.as_any_mut();
        tables_any.downcast_mut::<K::Signals>()
    }
}
impl<T:Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

impl<T: ?Sized> Default for TypedMap<T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}
impl<T: ?Sized> Debug for TypedMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypedMap").finish()//not very descriptive
    }
}
impl ExampleDomain {
    pub fn get<T: Addition+'static>(&mut self) -> Option<AsViewMut<'_, T>> {
        let tables = self.tables.get_tables::<T>() ?;
        let solvers = self.solvers.get_solvers::<T>() ?;
        let scripts = self.scripts.get_scripts::<T>() ?;
        let signals = self.signals.get_signals::<T>() ?;

        let view = AsViewMut::<T>::new(tables, solvers, scripts, signals);
        Some(view)
    }
    pub fn add<T: Addition + 'static>(&mut self) -> Result<AsViewMut<'_,T>, ()> {
        let tables = T::make_tables();
        let boxed = Box::new(tables);
        self.tables.insert::<T>(boxed);

        let solvers = T::make_solvers();
        let boxed = Box::new(solvers);
        self.solvers.insert::<T>(boxed);

        let scripts = T::make_scripts();
        let boxed = Box::new(scripts);
        self.scripts.insert::<T>(boxed);

        let signals = T::make_signals();
        let boxed = Box::new(signals);
        self.signals.insert::<T>(boxed);

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
trait Tables: AsAny+Any {}
trait Solvers: AsAny+Any {}
trait Solver: AsAny+Any {}
impl<T: Solver> Solvers for T {}

trait Scripts: AsAny+Any {}
trait Signals: AsAny+Any {}
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

    use crate::{addition::*, ecs::{class::Class, class_strategy::GrowthStrategy}};

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
                hats: Class::new(GrowthStrategy::quart_kib::<u32>())
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

        let cowboy_tables = domain.tables.get_tables::<CowboyWorld>()
            .expect("Expect cowboyworld to exist by now.");
    }
}
