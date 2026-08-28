use std::{any::TypeId, collections::HashMap};

use crate::addition::{Addition, Tables as AdditionTables, Polysystem};
use crate::ecs::{ClassId, partition::View};

#[derive(Default)]
pub struct Scope {
    pub additions: HashMap<TypeId, Box<dyn View>>,
}

impl Scope {
    pub fn view<T: Addition>(&mut self) -> Option<&mut T::View> {
        let view_id = TypeId::of::<T>();
        let view_any = self.additions.get_mut(&view_id)?;
        view_any.as_any_mut().downcast_mut::<T::View>()
    }

    pub(crate) fn width(&self) -> usize {
        self.additions.values().map(|v| v.width()).sum()
    }

    pub(crate) fn matches<W: Addition>(
        &self,
        class_id: ClassId,
        tables: &Polysystem<dyn AdditionTables, W::Tables>,
    ) -> bool {
        for (addition_id, view) in &self.additions {
            let tables_any: &dyn AdditionTables = if *addition_id == TypeId::of::<W>() {
                &tables.core
            } else {
                let Some(t) = tables.get_t(*addition_id) else {
                    return false;
                };
                t
            };
            if !view.matches(class_id, tables_any) {
                return false;
            }
        }
        true
    }

    pub(crate) fn commit<W: Addition>(
        &mut self,
        class_id: ClassId,
        tables: &mut Polysystem<dyn AdditionTables, W::Tables>,
    ) -> Option<usize> {
        let mut row = None;
        for (addition_id, view) in &mut self.additions {
            if *addition_id == TypeId::of::<W>() {
                let tables_any: &mut dyn AdditionTables = &mut tables.core;
                let view_row = view.commit(class_id, tables_any);
                if row.is_none() {
                    row = view_row;
                }
            } else if let Some(tables_any) = tables.get_t_mut(*addition_id) {
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
