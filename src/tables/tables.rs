use std::{any::{Any, TypeId}, collections::HashMap};

use crate::tables::{core::CoreAddition, partition::Addition, system::SystemAddition};

pub struct Tables {
    pub core: CoreAddition,
    pub additions: HashMap<TypeId, Box<dyn Addition>>,
    pub system: SystemAddition,
}

pub struct TablesAdditions<'a> {
    inner: &'a mut HashMap<TypeId, Box<dyn Addition>>,
}

impl<'a> TablesAdditions<'a> {
    pub fn new(inner: &'a mut HashMap<TypeId, Box<dyn Addition>>) -> Self {
        Self { inner }
    }

    pub fn get<T: Addition + 'static>(&self) -> Option<&T> {
        self.inner.get(&TypeId::of::<T>()).and_then(|a| {
            let any: &dyn Any = a.as_ref();
            any.downcast_ref::<T>()
        })
    }

    pub fn get_mut<T: Addition + 'static>(&mut self) -> Option<&mut T> {
        self.inner.get_mut(&TypeId::of::<T>()).and_then(|a| {
            let any: &mut dyn Any = a.as_mut();
            any.downcast_mut::<T>()
        })
    }

    pub fn get_many_mut<T, K>(&mut self) -> Option<(&mut T, &mut K)>
    where
        T: Addition + 'static,
        K: Addition + 'static,
    {
        let a_id = TypeId::of::<T>();
        let b_id = TypeId::of::<K>();
        if a_id == b_id {
            return None;
        }

        let [Some(a), Some(b)] = self.inner.get_disjoint_mut([&a_id, &b_id]) else {
            return None;
        };

        let a: &mut dyn Any = a.as_mut();
        let b: &mut dyn Any = b.as_mut();
        Some((a.downcast_mut::<T>()?, b.downcast_mut::<K>()?))
    }
}

impl Tables {
    pub fn get<T: Addition + 'static>(&self) -> Option<&T> {
        self.additions.get(&TypeId::of::<T>()).and_then(|a| {
            let any: &dyn Any = a.as_ref();
            any.downcast_ref::<T>()
        })
    }

    pub fn get_mut<T: Addition + 'static>(&mut self) -> Option<&mut T> {
        self.additions.get_mut(&TypeId::of::<T>()).and_then(|a| {
            let any: &mut dyn Any = a.as_mut();
            any.downcast_mut::<T>()
        })
    }

    pub fn add<T: Addition + 'static>(&mut self, addition: T) {
        let id = TypeId::of::<T>();
        self.additions.insert(id, Box::new(addition));
    }

    pub fn remove<T: Addition + 'static>(&mut self) -> Option<T> {
        self.additions
            .remove(&TypeId::of::<T>())
            .and_then(|a| {
                let any: Box<dyn Any> = a;
                any.downcast::<T>().ok().map(|b| *b)
            })
    }
}