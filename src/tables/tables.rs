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
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|a| (a.as_ref() as &dyn Any).downcast_ref::<T>())
    }

    pub fn get_mut<T: Addition + 'static>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|a| (a.as_mut() as &mut dyn Any).downcast_mut::<T>())
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

        let a = (a.as_mut() as &mut dyn Any).downcast_mut::<T>()?;
        let b = (b.as_mut() as &mut dyn Any).downcast_mut::<K>()?;
        Some((a, b))
    }
}

impl Tables {
    pub fn get<T: Addition + 'static>(&self) -> Option<&T> {
        self.additions
            .get(&TypeId::of::<T>())
            .and_then(|a| (a.as_ref() as &dyn Any).downcast_ref::<T>())
    }
}