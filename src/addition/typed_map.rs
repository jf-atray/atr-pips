use std::{any::TypeId, collections::HashMap, fmt::Debug};

use downcast_rs::Downcast;

pub struct Polysystem<T: ?Sized, K> {
    pub core: K,
    pub pile: Polypile<T>,
}
pub struct Polypile<T: ?Sized>(HashMap<TypeId, Box<T>>);

impl<T: ?Sized, K> Polysystem<T, K> {
    pub fn new(core: K) -> Self {
        Self {
            core,
            pile: Polypile::new(),
        }
    }
}
impl<T: ?Sized> Polypile<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn insert<L: 'static>(&mut self, value: Box<T>) {
        let id = TypeId::of::<L>();
        self.0.insert(id, value);
    }
}

impl<T: ?Sized + Downcast, K> Polysystem<T, K> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let one = std::iter::once(&mut self.core);
        one.chain(self.pile.0.values_mut().map(|b| b.as_mut()))
    }
}
impl<T: ?Sized + Downcast> Polypile<T> {
    pub fn get_mut<L: 'static, U: 'static>(&mut self) -> Option<&mut U> {
        let id = TypeId::of::<L>();
        let value = self.0.get_mut(&id)?;
        let inner = value.as_mut();
        inner.as_any_mut().downcast_mut::<U>()
    }
}

impl<T: ?Sized + Debug, K: Debug> Debug for Polysystem<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedMap")
            .field("core", &self.core)
            .field("rest", &self.pile.0.len())
            .finish()
    }
}
