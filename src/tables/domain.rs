use std::{any::Any, collections::HashMap};

use slotmap::SlotMap;

use crate::tables::{ClassId, ClassRowPtr, PipId, scope::{Scope, Maker}, tables::Tables, partition::Addition};

pub struct Domain {
    pub tables: Tables,
    pub by_width: HashMap<usize, Vec<ClassId>>,
    pub heading: SlotMap<ClassId, ()>,
    pub ids: SlotMap<PipId, ClassRowPtr>,
}

impl Domain {
    pub fn new(tables: Tables) -> Self {
        Self {
            tables,
            by_width: HashMap::new(),
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
            let view_id = (view.as_ref() as &dyn Any).type_id();
            scope.additions.insert(view_id, (*addition_id, view));
        }

        maker.make_into(&mut scope);
        scope.system.pip_id = Some(pip);

        let width = scope.width();
        let candidates: Vec<ClassId> = self.by_width
            .get(&width)
            .map(|v| v.clone())
            .unwrap_or_default();

        let mut class_id = None;
        for id in candidates {
            if scope.matches(id, &self.tables) {
                class_id = Some(id);
                break;
            }
        }

        let class_id = class_id.unwrap_or_else(|| {
            let id = self.heading.insert(());
            self.by_width.entry(width).or_default().push(id);
            id
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