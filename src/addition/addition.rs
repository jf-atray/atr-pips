use super::domain::{TablesMap, SolversMap, ScriptsMap, SignalsMap};
use super::traits::{Tables, Solvers, Scripts, Signals};


pub trait Addition: Sized + 'static {
    type Tables: Tables;
    type Solvers: Solvers;
    type Scripts: Scripts;
    type Signals: Signals;
    fn make_tables() -> Self::Tables;
    fn make_solvers() -> Self::Solvers;
    fn make_scripts() -> Self::Scripts;
    fn make_signals() -> Self::Signals;

    fn tables(map: &mut TablesMap) -> Option<&mut Self::Tables> {
        map.get_mut::<Self, Self::Tables>()
    }
    fn solvers(map: &mut SolversMap) -> Option<&mut Self::Solvers> {
        map.get_mut::<Self, Self::Solvers>()
    }
    fn scripts(map: &mut ScriptsMap) -> Option<&mut Self::Scripts> {
        map.get_mut::<Self, Self::Scripts>()
    }
    fn signals(map: &mut SignalsMap) -> Option<&mut Self::Signals> {
        map.get_mut::<Self, Self::Signals>()
    }
}
