use std::{any::{Any, TypeId}, collections::HashMap};

use crate::tables::{ClassId, core::CoreView, system::SystemView, tables::Tables, partition::View};

#[derive(Default)]
pub struct Scope {
    pub core: CoreView,
    pub system: SystemView,
    pub additions: HashMap<TypeId, (TypeId, Box<dyn View>)>,
}

impl Scope {
    pub fn view<T: View>(&mut self) -> Option<&mut T> {
        let view_id = TypeId::of::<T>();
        let (_, view_any) = self.additions.get_mut(&view_id)?;
        view_any.as_any_mut().downcast_mut::<T>()
    }

    pub(crate) fn width(&self) -> usize {
        let mut n = self.core.width() + self.system.width();
        for (_, view) in self.additions.values() {
            n += view.width();
        }
        n
    }

    pub(crate) fn matches(&self, class_id: ClassId, tables: &Tables) -> bool {
        if !self.core.matches(class_id, &tables.core) {
            return false;
        }
        if !self.system.matches(class_id, &tables.system) {
            return false;
        }
        for (_view_id, (addition_id, view)) in &self.additions {
            let Some(addition) = tables.get_any(addition_id) else {
                return false;
            };
            if !view.matches(class_id, addition) {
                return false;
            }
        }
        true
    }

    pub(crate) fn commit(&mut self, class_id: ClassId, tables: &mut Tables) -> Option<usize> {
        let mut row = self.core.commit(class_id, &mut tables.core);
        let system_row = self.system.commit(class_id, &mut tables.system);
        if row.is_none() { row = system_row; }

        //todo check the dependency inversion on this.
        for (_view_id, (addition_id, view)) in &mut self.additions {
            if let Some(addition) = tables.get_any_mut(addition_id) {
                let view_row = view.commit(class_id, addition.as_mut());
                if row.is_none() { row = view_row; }
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
