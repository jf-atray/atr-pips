use std::{any::{Any, TypeId}, collections::HashMap};

use crate::tables::{ClassId, core::CoreView, tables::Tables, partition::View};

#[derive(Default)]
pub struct Scope {
    pub core: CoreView,
    pub additions: HashMap<TypeId, (TypeId, Box<dyn View>)>,
}

impl Scope {
    pub fn view<T: View>(&mut self) -> Option<&mut T> {
        let view_id = TypeId::of::<T>();
        let view_any = self.additions
            .get_mut(&view_id)?
            .1
            .as_mut() as &mut dyn Any;
        view_any.downcast_mut::<T>()
    }

    pub(crate) fn width(&self) -> usize {
        let mut n = self.core.width();
        for (_, view) in self.additions.values() {
            n += view.width();
        }
        n
    }

    pub(crate) fn matches(&self, class_id: ClassId, tables: &Tables) -> bool {
        if !self.core.matches(class_id, &tables.core as &dyn Any) {
            return false;
        }
        for (addition_id, view) in self.additions.values() {
            let Some(addition) = tables.additions.get(addition_id) else {
                return false;
            };
            if !view.matches(class_id, addition.as_ref() as &dyn Any) {
                return false;
            }
        }
        true
    }

    pub(crate) fn commit(&mut self, class_id: ClassId, tables: &mut Tables) -> Option<usize> {
        let mut row = self.core.commit(class_id, &mut tables.core as &mut dyn Any);

        for (addition_id, view) in self.additions.values_mut() {
            if let Some(addition) = tables.additions.get_mut(addition_id) {
                let view_row = view.commit(class_id, addition.as_mut() as &mut dyn Any);
                if row.is_none() { row = view_row; }
            }
        }

        row
    }
}


pub trait Maker: Any {
    fn make_into(&mut self, scope: &mut Scope);
}
