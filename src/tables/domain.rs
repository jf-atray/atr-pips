use std::{any::Any, collections::HashMap};

use slotmap::SlotMap;

use crate::tables::{ClassId, ClassRowPtr, scope::{Scope, Maker}, tables::Tables, partition::Addition};

pub struct Domain {
    pub tables: Tables,
    pub by_width: HashMap<usize, Vec<ClassId>>,
    pub heading: SlotMap<ClassId, ()>,
}

impl Domain {
    pub fn make(&mut self, maker: &mut dyn Maker) -> ClassRowPtr {
        let mut scope = Scope::default();

        for (addition_id, addition) in &self.tables.additions {
            let view = addition.view_default();
            let view_id = (view.as_ref() as &dyn Any).type_id();
            scope.additions.insert(view_id, (*addition_id, view));
        }

        maker.make_into(&mut scope);

        let width = scope.width();
        let candidates: Vec<ClassId> = self.by_width
            .get(&width)
            .map(|v| v.to_vec())
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

    pub fn destroy(&mut self, ptr: &ClassRowPtr) {
        self.tables.core.destroy(ptr.class_id, ptr.row_idx);

        for addition in self.tables.additions.values_mut() {
            addition.destroy(ptr.class_id, ptr.row_idx);
        }
    }
}