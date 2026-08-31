use std::collections::HashMap;

use slotmap::SlotMap;

use crate::anims::{AnimLibId, AnimationLibrary};
use crate::assets::SpriteEntry;
use crate::diagnostics::DiagnosticsAdd;
use crate::ecs::core::CoreAdd;
use crate::ecs::{ClassId, ClassRowPtr, PipId, scope::{Maker, Scope}};
use crate::ecs::partition::Partition;
use crate::input::Input;

use super::core::Addition;
use super::traits::{Tables, Solvers, Scripts, Signals};
use super::typed_map::Polysystem;
use super::view::AsViewMut;

#[derive(Debug)]
pub struct Pips {
    pub tables: TablesMap,
    pub pip_ids: crate::ecs::class::Class<PipId>,
    pub ids: Ids,
    pub anim_libs: AnimLibs,
    pub heading: SlotMap<ClassId, usize>,
}

impl Pips {
    pub fn clear(&mut self) {
        self.ids.clear();
        self.heading.clear();
        self.anim_libs.clear();
        self.pip_ids.data.values_mut().for_each(|c| c.clear());
        self.tables.core.clear();
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
            .pip_ids
            .data
            .get(ptr.class_id)
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
        self.tables.core.destroy(ptr.class_id, ptr.row_idx);
        for tables in self.tables.iter_mut() {
            tables.destroy(ptr.class_id, ptr.row_idx);
        }
        if let Some(col) = self.pip_ids.data.get_mut(ptr.class_id) {
            col.swap_remove(ptr.row_idx);
        }
    }

    fn commit_with<F: FnOnce(&mut Scope)>(&mut self, pip: PipId, f: F) -> ClassRowPtr {
        let mut scope = Scope::default();

        for tables in self.tables.iter_mut() {
            let view = tables.view_default();
            scope.additions.insert(view.addition_id(), view);
        }

        f(&mut scope);

        let width = scope.width();

        let mut class_id = None;
        for id in self
            .heading
            .iter()
            .filter(|(_, v)| **v == width)
            .map(|(k, _)| k)
        {
            if scope.matches::<CoreAdd>(id, &self.tables) {
                class_id = Some(id);
                break;
            }
        }

        let class_id = class_id.unwrap_or_else(|| self.heading.insert(width));
        let row_idx = scope.commit::<CoreAdd>(class_id, &mut self.tables).unwrap();
        self.pip_ids.get_col_or_insert(class_id).push(pip);
        ClassRowPtr::new(class_id, row_idx)
    }
}

#[derive(Debug)]
pub struct ExampleDomain {
    pub pips: Pips,
    pub solvers: SolversMap,
    pub scripts: ScriptsMap,
    pub signals: SignalsMap,
}

pub type TablesMap = Polysystem<dyn Tables, <CoreAdd as Addition>::Tables>;
pub type SolversMap = Polysystem<dyn Solvers, <CoreAdd as Addition>::Solvers>;
pub type ScriptsMap = Polysystem<dyn Scripts, <CoreAdd as Addition>::Scripts>;
pub type SignalsMap = Polysystem<dyn Signals, <CoreAdd as Addition>::Signals>;
pub type Ids = SlotMap<PipId, ClassRowPtr>;
pub type AnimLibs = SlotMap<AnimLibId, AnimationLibrary>;

impl Default for ExampleDomain {
    fn default() -> Self {
        let mut domain = Self {
            pips: Pips {
                tables: TablesMap::new(CoreAdd::make_tables()),
                pip_ids: crate::ecs::class::Class::new(
                    crate::ecs::class_strategy::GrowthStrategy::quart_kib::<PipId>(),
                ),
                ids: Ids::default(),
                anim_libs: AnimLibs::default(),
                heading: SlotMap::default(),
            },
            solvers: SolversMap::new(CoreAdd::make_solvers()),
            scripts: ScriptsMap::new(CoreAdd::make_scripts()),
            signals: SignalsMap::new(CoreAdd::make_signals()),
        };

        domain
            .add::<DiagnosticsAdd>()
            .expect("DiagnosticsAdd must be available");

        domain
    }
}

impl ExampleDomain {
    pub fn get<T: Addition + 'static>(&mut self) -> Option<AsViewMut<'_, T>> {
        let tables = self.pips.tables.get_mut::<T, T::Tables>()?;
        let solvers = self.solvers.get_mut::<T, T::Solvers>()?;
        let scripts = self.scripts.get_mut::<T, T::Scripts>()?;
        let signals = self.signals.get_mut::<T, T::Signals>()?;

        let view = AsViewMut::<T>::new(tables, solvers, scripts, signals);
        Some(view)
    }

    pub fn add<T: Addition + 'static>(&mut self) -> Result<AsViewMut<'_, T>, ()> {
        self.pips.tables.insert::<T>(Box::new(T::make_tables()));
        self.solvers.insert::<T>(Box::new(T::make_solvers()));
        self.scripts.insert::<T>(Box::new(T::make_scripts()));
        self.signals.insert::<T>(Box::new(T::make_signals()));

        self.get::<T>().ok_or(())
    }

    pub fn update_solvers(
        &mut self,
        dt: f32,
        input: &mut Input,
        asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        self.solvers.core.update(
            dt,
            &mut self.pips,
            &mut self.scripts,
            &mut self.signals,
            input,
            asset_registry,
        );
        for solver in self.solvers.iter_mut() {
            solver.update(
                dt,
                &mut self.pips,
                &mut self.scripts,
                &mut self.signals,
                input,
                asset_registry,
            );
        }
    }

    pub fn clear(&mut self) {
        self.pips.clear();
    }

    pub fn make<M: Maker>(&mut self, maker: M) -> PipId {
        self.pips.make(maker)
    }

    pub fn destroy(&mut self, pip: PipId) {
        self.pips.destroy(pip);
    }
}
