use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::ecs::{core::CoreAddition, partition::Addition, system::SystemAddition};

pub struct Tables {
    pub core: CoreAddition,
    pub system: SystemAddition,
    additions: HashMap<TypeId, Box<dyn Addition>>,
}
impl Tables {
    pub fn with(core: CoreAddition, system: SystemAddition) -> Self {
        Self {
            core,
            system,
            additions: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        Self::with(CoreAddition::new(), SystemAddition::new())
    }

    pub fn view(&mut self) -> TablesView<'_> {
        let additions = AdditionsView::new(&mut self.additions);
        TablesView {
            core: &mut self.core,
            system: &mut self.system,
            additions,
        }
    }
    pub fn clear(&mut self) {
        self.core.clear();
        self.system.clear();
        for adtn in &mut self.additions {
            adtn.1.clear();
        }
    }
}

pub struct TablesView<'a> {
    pub core: &'a mut CoreAddition,
    pub system: &'a mut SystemAddition,
    pub additions: AdditionsView<'a>,
}

pub struct AdditionsView<'a> {
    inner: &'a mut HashMap<TypeId, Box<dyn Addition>>,
}

impl<'a> AdditionsView<'a> {
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

    pub fn get_or_insert<T: Addition + 'static>(&mut self, default: T) -> &mut T {
        use std::collections::hash_map::Entry;
        match self.inner.entry(TypeId::of::<T>()) {
            Entry::Occupied(e) => {
                let any: &mut dyn Any = e.into_mut().as_mut();
                any.downcast_mut::<T>().expect("addition type mismatch")
            }
            Entry::Vacant(e) => {
                let any: &mut dyn Any = e.insert(Box::new(default)).as_mut();
                any.downcast_mut::<T>().expect("addition type mismatch")
            }
        }
    }

    pub fn disjoin<const N: usize>(
        &mut self,
        ids: [TypeId; N],
    ) -> Option<[&mut Box<dyn Addition>; N]> {
        let refs: [&TypeId; N] = std::array::from_fn(|i| &ids[i]);
        let results: [Option<&mut Box<dyn Addition>>; N] = self.inner.get_disjoint_mut(refs);
        if results.iter().any(std::option::Option::is_none) {
            //I think it's just simpler if it's all or nothing
            return None;
        }
        Some(results.map(|o| o.unwrap()))
    }

    pub fn get_both_mut<T, K>(&mut self) -> Option<(&mut T, &mut K)>
    where
        T: Addition + 'static,
        K: Addition + 'static,
    {
        let ids = [TypeId::of::<T>(), TypeId::of::<K>()];
        let [a, b] = self.disjoin(ids)?;

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

    pub fn get_or_insert<T: Addition + 'static>(&mut self, default: T) -> &mut T {
        use std::collections::hash_map::Entry;
        match self.additions.entry(TypeId::of::<T>()) {
            Entry::Occupied(e) => {
                let any: &mut dyn Any = e.into_mut().as_mut();
                any.downcast_mut::<T>().expect("addition type mismatch")
            }
            Entry::Vacant(e) => {
                let any: &mut dyn Any = e.insert(Box::new(default)).as_mut();
                any.downcast_mut::<T>().expect("addition type mismatch")
            }
        }
    }
    pub fn get_any(&self, id: &TypeId) -> Option<&dyn Addition> {
        self.additions.get(id).map(std::convert::AsRef::as_ref)
    }
    pub fn get_any_mut(&mut self, id: &TypeId) -> Option<&mut Box<dyn Addition + 'static>> {
        self.additions.get_mut(id)
    }

    pub fn addition_entries(&self) -> impl Iterator<Item = (&TypeId, &dyn Addition)> {
        self.additions.iter().map(|(id, a)| (id, a.as_ref()))
    }

    pub fn addition_values_mut(&mut self) -> impl Iterator<Item = &mut dyn Addition> {
        self.additions.values_mut().map(std::convert::AsMut::as_mut)
    }

    pub fn add<T: Addition + 'static>(&mut self, addition: T) {
        let id = TypeId::of::<T>();
        self.additions.insert(id, Box::new(addition));
    }

    pub fn remove<T: Addition + 'static>(&mut self) -> Option<T> {
        self.additions.remove(&TypeId::of::<T>()).and_then(|a| {
            let any: Box<dyn Any> = a;
            any.downcast::<T>().ok().map(|b| *b)
        })
    }
}
