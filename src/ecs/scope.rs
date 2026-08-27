use std::{any::TypeId, collections::HashMap};

use crate::addition::{TypedMap, Tables as AdditionTables};
use crate::ecs::{ClassId, partition::View};

#[derive(Default)]
pub struct Scope {
    pub additions: HashMap<TypeId, Box<dyn View>>,
}

impl Scope {
    pub fn view<T: View>(&mut self) -> Option<&mut T> {
        let view_id = TypeId::of::<T>();
        let view_any = self.additions.get_mut(&view_id)?;
        view_any.as_any_mut().downcast_mut::<T>()
    }

    pub(crate) fn width(&self) -> usize {
        self.additions.values().map(|v| v.width()).sum()
    }

    pub(crate) fn matches(&self, class_id: ClassId, tables: &TypedMap<dyn AdditionTables>) -> bool {
        for (addition_id, view) in &self.additions {
            let Some(tables_any) = tables.get_dyn(addition_id) else {
                return false;
            };
            if !view.matches(class_id, tables_any) {
                return false;
            }
        }
        true
    }

    pub(crate) fn commit(
        &mut self,
        class_id: ClassId,
        tables: &mut TypedMap<dyn AdditionTables>,
    ) -> Option<usize> {
        let mut row = None;
        for (addition_id, view) in &mut self.additions {
            if let Some(tables_any) = tables.get_dyn_mut(addition_id) {
                let view_row = view.commit(class_id, tables_any);
                if row.is_none() {
                    row = view_row;
                }
            }
        }
        row
    }
}

pub trait Maker {
    fn make_into(self, scope: &mut Scope);
}

impl<F: FnOnce(&mut Scope)> Maker for F {
    fn make_into(self, scope: &mut Scope) {
        self(scope);
    }
}
