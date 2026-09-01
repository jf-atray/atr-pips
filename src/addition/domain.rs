use std::collections::HashMap;

use slotmap::SlotMap;

use crate::anims::{AnimLibId, AnimationLibrary};
use crate::assets::SpriteEntry;
use crate::diagnostics::DiagnosticsAdd;
use crate::ecs::core::CoreAdd;
use crate::ecs::{ClassId, ClassRowPtr, PipId, scope::{Maker, Scope}};
use crate::ecs::partition::{Partition, View};
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
    pub scratch: Scope,
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
        let mut scope = self.take_scratch();
        maker.make_into(&mut scope);
        let ptr = self.commit_scope(pip, &mut scope);
        self.ids[pip] = ptr;
        self.scratch = scope;
        pip
    }

    pub fn destroy(&mut self, pip: PipId) {
        let Some(ptr) = self.ids.get(pip).cloned() else {
            return;
        };

        self.destroy_ptr(&ptr);
        self.fix_displaced(&ptr);
        self.ids.remove(pip);
    }

    pub fn extract_pip(&mut self, pip: PipId) -> Option<Scope> {
        let ptr = self.ids.get(pip).cloned()?;

        let mut scope = self.fresh_scope();
        scope.extract::<CoreAdd>(ptr.class_id, ptr.row_idx, &mut self.tables);

        if let Some(col) = self.pip_ids.data.get_mut(ptr.class_id) {
            col.swap_remove(ptr.row_idx);
        }
        self.fix_displaced(&ptr);
        self.ids.remove(pip);

        Some(scope)
    }

    pub fn move_pip<M: Maker>(&mut self, pip: PipId, maker: M) {
        let Some(old_ptr) = self.ids.get(pip).cloned() else {
            return;
        };

        let mut scope = self.take_scratch();
        scope.extract::<CoreAdd>(old_ptr.class_id, old_ptr.row_idx, &mut self.tables);

        if let Some(col) = self.pip_ids.data.get_mut(old_ptr.class_id) {
            col.swap_remove(old_ptr.row_idx);
        }
        self.fix_displaced(&old_ptr);

        maker.make_into(&mut scope);

        let new_ptr = self.commit_scope(pip, &mut scope);
        self.ids[pip] = new_ptr;
        self.scratch = scope;
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

    fn fix_displaced(&mut self, ptr: &ClassRowPtr) {
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
    }

    fn take_scratch(&mut self) -> Scope {
        let mut scope = std::mem::take(&mut self.scratch);

        for (id, tables) in self.tables.kvp_iter_mut() {
            if !scope.additions.contains_key(id) {
                let view = tables.view_default();
                scope.additions.insert(*id, view);
            }
        }

        scope.core.reset();
        for view in scope.additions.values_mut() {
            view.reset();
        }

        scope
    }

    fn fresh_scope(&mut self) -> Scope {
        let mut scope = Scope::default();
        for (id, tables) in self.tables.kvp_iter_mut() {
            let view = tables.view_default();
            scope.additions.insert(*id, view);
        }
        scope
    }

    fn commit_scope(&mut self, pip: PipId, scope: &mut Scope) -> ClassRowPtr {
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
                scratch: Scope::default(),
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

    pub fn extract_pip(&mut self, pip: PipId) -> Option<Scope> {
        self.pips.extract_pip(pip)
    }

    pub fn move_pip<M: Maker>(&mut self, pip: PipId, maker: M) {
        self.pips.move_pip(pip, maker);
    }
}
