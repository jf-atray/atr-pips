use std::{any::TypeId, collections::HashMap};

use crate::addition::{Addition, Polysystem, Tables as AdditionTables};
use crate::ecs::{ClassId, core::CoreAdd, partition::View};

#[derive(Default)]
pub struct Scope {
    pub core: <CoreAdd as Addition>::View,
    pub additions: HashMap<TypeId, Box<dyn View>>,
}

impl Scope {
    pub fn view<T: Addition>(&mut self) -> Option<&mut T::View> {
        let view_any = self.additions.get_mut(&TypeId::of::<T>())?;
        view_any.as_any_mut().downcast_mut::<T::View>()
    }

    pub(crate) fn width(&self) -> usize {
        self.core.width() + self.additions.values().map(|v| v.width()).sum::<usize>()
    }

    pub(crate) fn matches<W: Addition>(
        &self,
        class_id: ClassId,
        tables: &Polysystem<dyn AdditionTables, W::Tables>,
    ) -> bool {
        if !self.core.matches(class_id, &tables.core) {
            return false;
        }
        for (addition_id, view) in &self.additions {
            let Some(tables_any) = tables.get_t(*addition_id) else {
                return false;
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
        let mut row = self.core.commit(class_id, &mut tables.core);
        for (addition_id, view) in &mut self.additions {
            if let Some(tables_any) = tables.get_t_mut(*addition_id) {
                row = row.or(view.commit(class_id, tables_any));
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
