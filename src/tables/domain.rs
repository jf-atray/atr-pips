use std::{any::Any, collections::HashMap};

use slotmap::SlotMap;

use crate::tables::{ClassId, ClassRowPtr, PipId, scope::{Scope, Maker}, tables::Tables, partition::Addition};

pub struct Domain {
    pub tables: Tables,
    pub heading: SlotMap<ClassId, usize>,
    pub ids: SlotMap<PipId, ClassRowPtr>,
}

impl Domain {
    pub fn new(tables: Tables) -> Self {
        Self {
            tables,
            heading: SlotMap::with_key(),
            ids: SlotMap::with_key(),
        }
    }

    pub fn make<M: Maker>(&mut self, maker: M) -> PipId {
        //acquires a generational index
        let pip = self.ids.insert(ClassRowPtr::new(ClassId::default(), 0));
        let ptr = self.commit(pip, maker);
        //backfill with what class we actually discovered
        self.ids[pip] = ptr;
        pip
    }

    pub fn destroy(&mut self, pip: PipId) {
        let Some(ptr) = self.ids.get(pip).cloned() else {
            return;
        };

        self.destroy_ptr(&ptr);
        let displaced = self.tables.system.pip_id
            .get_col(ptr.class_id)
            .and_then(|col| col.get(ptr.row_idx))
            .copied();

        if let Some(displaced) = displaced
            && let Some(entry) = self.ids.get_mut(displaced) {
                *entry = ClassRowPtr::new(ptr.class_id, ptr.row_idx);
            }

        self.ids.remove(pip);
    }



    fn commit<M: Maker>(&mut self, pip: PipId, maker: M) -> ClassRowPtr {
        let mut scope = Scope::default();

        for (addition_id, addition) in &self.tables.additions {
            let view = addition.view_default();
            scope.additions.insert(*addition_id, view);
        }

        maker.make_into(&mut scope);
        scope.system.pip_id = Some(pip);

        let width = scope.width();


        let mut class_id = None;
        //todo in youfirst I actually itered the smallest commited class
        //but depending on the kind of slotmap this might actually be
        //less than useful.
        //doublecheck secondarymap because if sparce is still o(1) for a little more memory
        //and gains a fwd walk key iter... yummy
        for id in self.heading.iter().filter(|(_,v)| **v == width ).map(|(k,_)| k){
            if scope.matches(id, &self.tables) {
                class_id = Some(id);
                break;
            }
        }

        let class_id = class_id.unwrap_or_else(|| {
            self.heading.insert(width)
        });

        let row_idx = scope.commit(class_id, &mut self.tables).unwrap();
        ClassRowPtr::new(class_id, row_idx)
    }

    fn destroy_ptr(&mut self, ptr: &ClassRowPtr) {
        self.tables.core.destroy(ptr.class_id, ptr.row_idx);
        self.tables.system.destroy(ptr.class_id, ptr.row_idx);

        for addition in self.tables.additions.values_mut() {
            addition.destroy(ptr.class_id, ptr.row_idx);
        }
    }
}