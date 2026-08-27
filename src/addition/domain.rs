use slotmap::SlotMap;

use crate::anims::{AnimLibId, AnimationLibrary};
use crate::ecs::core::CoreWorld;
use crate::ecs::{ClassId, ClassRowPtr, PipId, scope::{Maker, Scope}, system::{SystemAddition, SystemView, SystemWorld}};

use super::addition::Addition;
use super::traits::{Tables, Solvers, Scripts, Signals};
use super::typed_map::TypedMap;
use super::view::AsViewMut;

#[derive(Default, Debug)]
pub struct ExampleDomain {
    pub tables: TablesMap,
    pub solvers: SolversMap,
    pub scripts: ScriptsMap,
    pub signals: SignalsMap,
    pub anim_libs: SlotMap<AnimLibId, AnimationLibrary>,
    pub heading: SlotMap<ClassId, usize>,
    pub ids: SlotMap<PipId, ClassRowPtr>,
}
pub type TablesMap = TypedMap<dyn Tables, <CoreWorld as Addition>::Tables>;
pub type SolversMap = TypedMap<dyn Solvers, <CoreWorld as Addition>::Solvers>;
pub type ScriptsMap = TypedMap<dyn Scripts, <CoreWorld as Addition>::Scripts>;
pub type SignalsMap = TypedMap<dyn Signals, <CoreWorld as Addition>::Signals>;

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

    pub fn update_solvers(&mut self, dt: f32) {
        for solver in self.solvers.iter_mut() {
            solver.update(dt, &mut self.tables, &mut self.scripts, &mut self.signals);
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.heading.clear();
        self.anim_libs.clear();
        for tables in self.tables.iter_mut() {
            tables.clear();
        }
    }

    pub fn make<M: Maker>(&mut self, maker: M) -> PipId {
        let pip = self.ids.insert(ClassRowPtr::new(ClassId::default(), 0));
        let ptr = self.commit_with(pip, |scope| maker.make_into(scope));
        self.ids[pip] = ptr;
        pip
    }

    pub fn destroy(&mut self, pip: PipId) {
        let Some(ptr) = self.ids.get(pip).cloned() else {
            return;
        };

        self.destroy_ptr(&ptr);
        let displaced = self
            .tables
            .get_mut::<SystemWorld, SystemAddition>()
            .and_then(|sys| sys.pip_id.data.get(ptr.class_id))
            .and_then(|col| col.get(ptr.row_idx))
            .copied();

        if let Some(displaced) = displaced
            && let Some(entry) = self.ids.get_mut(displaced)
        {
            *entry = ClassRowPtr::new(ptr.class_id, ptr.row_idx);
        }

        self.ids.remove(pip);
    }

    fn destroy_ptr(&mut self, ptr: &ClassRowPtr) {
        for tables in self.tables.iter_mut() {
            tables.destroy(ptr.class_id, ptr.row_idx);
        }
    }

    fn commit_with<F: FnOnce(&mut Scope)>(&mut self, pip: PipId, f: F) -> ClassRowPtr {
        let mut scope = Scope::default();

        for tables in self.tables.iter_mut() {
            let view = tables.view_default();
            let view_id = view.as_any_ref().type_id();
            scope.additions.insert(view_id, view);
        }

        f(&mut scope);
        if let Some(sys_view) = scope.view::<SystemView>() {
            sys_view.pip_id = Some(pip);
        }

        let width = scope.width();

        let mut class_id = None;
        for id in self
            .heading
            .iter()
            .filter(|(_, v)| **v == width)
            .map(|(k, _)| k)
        {
            if scope.matches(id, &self.tables) {
                class_id = Some(id);
                break;
            }
        }

        let class_id = class_id.unwrap_or_else(|| self.heading.insert(width));
        let row_idx = scope.commit(class_id, &mut self.tables).unwrap();
        ClassRowPtr::new(class_id, row_idx)
    }
}
