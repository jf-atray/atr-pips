use std::{any::TypeId, collections::HashMap, fmt::Debug};

use downcast_rs::Downcast;

pub struct TypedMap<T: ?Sized, K> {
    pub core: K,
    rest: HashMap<TypeId, Box<T>>,
}

impl<T: ?Sized, K> TypedMap<T, K> {
    pub fn new(core: K) -> Self {
        Self {
            core,
            rest: HashMap::new(),
        }
    }

    pub(crate) fn insert<L: 'static>(&mut self, value: Box<T>) {
        let id = TypeId::of::<L>();
        self.rest.insert(id, value);
    }
}

impl<T: ?Sized + Downcast, K> TypedMap<T, K> {
    pub fn get_mut<L: 'static, U: 'static>(&mut self) -> Option<&mut U> {
        let id = TypeId::of::<L>();
        let value = self.rest.get_mut(&id)?;
        let inner = value.as_mut();
        inner.as_any_mut().downcast_mut::<U>()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let one = std::iter::once(&mut self.core);
        one.chain(self.rest.values_mut().map(|b| b.as_mut()))
    }
}

impl<T: ?Sized + Debug, K: T + Debug> Debug for TypedMap<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedMap")
            .field("core", &self.core)
            .field("rest", &self.rest.len())
            .finish()
    }
}
