use super::addition::Addition;
use super::traits::{Tables, Solvers, Scripts, Signals};
use super::typed_map::TypedMap;
use super::view::AsViewMut;

#[derive(Default, Debug)]
pub(super) struct ExampleDomain {
    pub tables: TypedMap<dyn Tables>,
    pub solvers: TypedMap<dyn Solvers>,
    pub scripts: TypedMap<dyn Scripts>,
    pub signals: TypedMap<dyn Signals>,
}

impl ExampleDomain {
    pub fn get<T: Addition + 'static>(&mut self) -> Option<AsViewMut<'_, T>> {
        let tables = self.tables.get_mut::<T, T::Tables>()?;
        let solvers = self.solvers.get_mut::<T, T::Solvers>()?;
        let scripts = self.scripts.get_mut::<T, T::Scripts>()?;
        let signals = self.signals.get_mut::<T, T::Signals>()?;

        let view = AsViewMut::<T>::new(tables, solvers, scripts, signals);
        Some(view)
    }

    pub fn add<T: Addition + 'static>(&mut self) -> Result<AsViewMut<'_, T>, ()> {
        self.tables.insert::<T>(Box::new(T::make_tables()));
        self.solvers.insert::<T>(Box::new(T::make_solvers()));
        self.scripts.insert::<T>(Box::new(T::make_scripts()));
        self.signals.insert::<T>(Box::new(T::make_signals()));

        self.get::<T>().ok_or(())
    }
}
