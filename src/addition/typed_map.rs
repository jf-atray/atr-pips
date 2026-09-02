use std::{any::TypeId, collections::HashMap, fmt::Debug};

use downcast_rs::Downcast;

pub struct Polysystem<T: ?Sized, K> {
    pub core: K,
    pub pile: Polypile<T>,
}
pub struct Polypile<T: ?Sized>(HashMap<TypeId, Box<T>>);

impl<T: ?Sized, K> AsRef<Polypile<T>> for Polysystem<T, K> {
    fn as_ref(&self) -> &Polypile<T> {
        &self.pile
    }
}
impl<T: ?Sized, K> AsMut<Polypile<T>> for Polysystem<T, K> {
    fn as_mut(&mut self) -> &mut Polypile<T> {
        &mut self.pile
    }
}

impl<T: ?Sized, K> Polysystem<T, K> {
    pub fn new(core: K) -> Self {
        Self {
            core,
            pile: Polypile::new(),
        }
    }

    pub fn insert<L: 'static>(&mut self, value: Box<T>) {
        self.pile.0.insert(TypeId::of::<L>(), value);
    }
}
impl<T: ?Sized> Polypile<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert<L: 'static>(&mut self, value: Box<T>) {
        let id = TypeId::of::<L>();
        self.0.insert(id, value);
    }
}

impl<T: ?Sized + Downcast, K> Polysystem<T, K> {
    pub fn get_mut<L: 'static, U: 'static>(&mut self) -> Option<&mut U> {
        self.pile.get_mut::<L, U>()
    }

    pub fn get_t(&self, id: TypeId) -> Option<&T> {
        self.pile.get_t(id)
    }

    pub fn get_t_mut(&mut self, id: TypeId) -> Option<&mut T> {
        self.pile.get_t_mut(id)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.pile.0.values_mut().map(AsMut::as_mut)
    }
    pub fn kvp_iter_mut(&mut self) -> impl Iterator<Item = (&TypeId, &mut T)> {
        self.pile.0.iter_mut().map(|i| (i.0, i.1.as_mut()))
    }
}
impl<T: ?Sized + Downcast> Polypile<T> {
    pub fn get<L: 'static, U: 'static>(&self) -> Option<&U> {
        let id = TypeId::of::<L>();
        let value = self.0.get(&id)?;
        let inner = value.as_ref();
        inner.as_any().downcast_ref::<U>()
    }

    pub fn get_mut<L: 'static, U: 'static>(&mut self) -> Option<&mut U> {
        let id = TypeId::of::<L>();
        let value = self.0.get_mut(&id)?;
        let inner = value.as_mut();
        inner.as_any_mut().downcast_mut::<U>()
    }

    pub fn get_t(&self, id: TypeId) -> Option<&T> {
        self.0.get(&id).map(std::convert::AsRef::as_ref)
    }

    pub fn get_t_mut(&mut self, id: TypeId) -> Option<&mut T> {
        self.0.get_mut(&id).map(std::convert::AsMut::as_mut)
    }
}

impl<T: ?Sized> AsRef<Polypile<T>> for Polypile<T> {
    fn as_ref(&self) -> &Polypile<T> {
        self
    }
}

impl<T: ?Sized> AsMut<Polypile<T>> for Polypile<T> {
    fn as_mut(&mut self) -> &mut Polypile<T> {
        self
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
