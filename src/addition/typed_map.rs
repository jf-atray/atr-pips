use std::{any::TypeId, collections::HashMap, fmt::Debug};

use downcast_rs::Downcast;

pub struct TypedMap<T: ?Sized>(HashMap<TypeId, Box<T>>);

impl<T: ?Sized> TypedMap<T> {
    //so sorry but people keep reaching into here. todo, make a wrapper struct for locked visibility
    pub(super) fn insert<K: 'static>(&mut self, value: Box<T>) {
        let id = TypeId::of::<K>();
        self.0.insert(id, value);
    }
}

impl<T: ?Sized + Downcast> TypedMap<T> {
    pub fn get_mut<K: 'static, U: 'static>(&mut self) -> Option<&mut U> {
        let id = TypeId::of::<K>();
        let value = self.0.get_mut(&id)?;
        let inner = value.as_mut();
        inner.as_any_mut().downcast_mut::<U>()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.values_mut().map(|b| b.as_mut())
    }
}


impl<T: ?Sized> Default for TypedMap<T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<T: ?Sized> Debug for TypedMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypedMap").finish()
    }
}
