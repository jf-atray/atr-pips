use slotmap::SlotMap;

use crate::anims::{AnimLibId, AnimationLibrary};
use crate::ecs::{
    ClassId, ClassRowPtr, PipId,
    partition::Addition,
    scope::{Maker, Scope},
    tables::Tables,
};

#[derive(Debug)]
pub struct Domain {
    pub tables: Tables,
    pub anim_libs: SlotMap<AnimLibId, AnimationLibrary>,
    pub heading: SlotMap<ClassId, usize>,
    pub ids: SlotMap<PipId, ClassRowPtr>,
}

impl Domain {
    pub fn new() -> Self {
        Self::with_tables(Tables::new())
    }

    pub fn with_tables(tables: Tables) -> Self {
        Self {
            tables,
            anim_libs: SlotMap::with_key(),
            heading: SlotMap::with_key(),
            ids: SlotMap::with_key(),
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.heading.clear();
        self.tables.clear();
        self.anim_libs.clear();
    }

    pub fn make<M: Maker>(&mut self, maker: M) -> PipId {
        //acquires a generational index
        let pip = self.ids.insert(ClassRowPtr::new(ClassId::default(), 0));
        let ptr = self.commit_with(pip, |scope| maker.make_into(scope));
        //backfill with what class we actually discovered
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
            .system
            .pip_id
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

    fn commit_with<F: FnOnce(&mut Scope)>(&mut self, pip: PipId, f: F) -> ClassRowPtr {
        let mut scope = Scope::default();

        for (addition_id, addition) in self.tables.addition_entries() {
            let view = addition.view_default();
            let view_id = view.as_any_ref().type_id();
            scope.additions.insert(view_id, (*addition_id, view));
        }

        f(&mut scope);
        scope.system.pip_id = Some(pip);

        let width = scope.width();

        let mut class_id = None;
        //todo in youfirst I actually itered the smallest commited class
        //but depending on the kind of slotmap this might actually be
        //less than useful.
        //doublecheck secondarymap because if sparce is still o(1) for a little more memory
        //and gains a fwd walk key iter... yummy
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

    fn destroy_ptr(&mut self, ptr: &ClassRowPtr) {
        self.tables.core.destroy(ptr.class_id, ptr.row_idx);
        self.tables.system.destroy(ptr.class_id, ptr.row_idx);

        for addition in self.tables.addition_values_mut() {
            addition.destroy(ptr.class_id, ptr.row_idx);
        }
    }
}
