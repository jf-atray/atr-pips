use super::traits::{Tables, Solvers, Scripts, Signals};
use crate::addition::typed_map::Polypile;
use crate::ecs::partition::View as ViewTrait;

pub trait Addition: Sized + 'static {
    type Tables: Tables;
    type Solvers: Solvers;
    type Scripts: Scripts;
    type Signals: Signals;
    type View: ViewTrait;
    fn make_tables() -> Self::Tables;
    fn make_solvers() -> Self::Solvers;
    fn make_scripts() -> Self::Scripts;
    fn make_signals() -> Self::Signals;

    fn tables<T: AsMut<Polypile<dyn Tables>>>(map: &mut T) -> Option<&mut Self::Tables> {
        let pile = map.as_mut();
        pile.get_mut::<Self, Self::Tables>()
    }
    fn tables_ref<T: AsRef<Polypile<dyn Tables>>>(map: &T) -> Option<&Self::Tables> {
        let pile = map.as_ref();
        pile.get::<Self, Self::Tables>()
    }
    fn solvers<T: AsMut<Polypile<dyn Solvers>>>(map: &mut T) -> Option<&mut Self::Solvers> {
        let pile = map.as_mut();
        pile.get_mut::<Self, Self::Solvers>()
    }
    fn scripts<T: AsMut<Polypile<dyn Scripts>>>(map: &mut T) -> Option<&mut Self::Scripts> {
        let pile = map.as_mut();
        pile.get_mut::<Self, Self::Scripts>()
    }
    fn signals<T: AsMut<Polypile<dyn Signals>>>(map: &mut T) -> Option<&mut Self::Signals> {
        let pile = map.as_mut();
        pile.get_mut::<Self, Self::Signals>()
    }
    fn signals_ref<T: AsRef<Polypile<dyn Signals>>>(map: &T) -> Option<&Self::Signals> {
        let pile = map.as_ref();
        pile.get::<Self, Self::Signals>()
    }
}
