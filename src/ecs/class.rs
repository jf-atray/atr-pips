use std::{collections::BTreeSet, ops::{Index, IndexMut}};

use atr_plex::duplex;
use slotmap::SecondaryMap;

use crate::ecs::{
    ClassId, ClassRowPtr,
    class_strategy::GrowthStrategy,
};

#[derive(Debug)]
pub struct Class<T, K = ()> {
    pub(crate) growth: GrowthStrategy,
    //todo, worry about excessive vec ptr memory bloat later
    pub(crate) data: SecondaryMap<ClassId, Columnar<T, K>>,
    pub(crate) keys: BTreeSet<ClassId>,

}
impl<T, K> Class<T, K> {
    pub(crate) fn new(growth: GrowthStrategy) -> Self {
        let data = SecondaryMap::new();
        Self { growth, data, keys: BTreeSet::new() }
    }
    pub(crate) fn stake(&mut self, class_id: ClassId, k: K) -> &mut Self{
        self.data.insert(class_id, Columnar::new(k));
        let _notdupe = self.keys.insert(class_id);
        self
    }
    pub(crate) fn with_capacity(capacity: usize, _rarity: duplex::Thin, growth: GrowthStrategy) -> Self {
        let data = SecondaryMap::with_capacity(capacity);
        Self { growth, data, keys: BTreeSet::new() }
    }

    pub(super) fn get_row(&self, id: &ClassRowPtr) -> Option<&T> {
        let col = self.data.get(id.class_id)?;
        Some(&col.vec[id.row_idx])
    }
    pub(super) fn get_row_mut(&mut self, id: &ClassRowPtr) -> Option<&mut T> {
        let col = self.data.get_mut(id.class_id)?;
        Some(&mut col.vec[id.row_idx])
    }

    pub(super) unsafe fn get_row_unchecked(&self, id: &ClassRowPtr) -> Option<&T> {
        let col = self.data.get(id.class_id)?;
        let row = unsafe { col.get_unchecked(id.row_idx) };
        Some(row)
    }
    pub(super) unsafe fn get_row_unchecked_mut(&mut self, id: &ClassRowPtr) -> Option<&mut T> {
        let col = self.data.get_mut(id.class_id)?;
        let row = unsafe { col.get_unchecked_mut(id.row_idx) };
        Some(row)
    }
}

impl<T, K: PartialEq> Class<T, K> {
    pub(crate) fn get_col_or_insert_with_key(&mut self, id: ClassId, k: K) -> &mut Columnar<T, K> {
        if self.data.contains_key(id) {
            debug_assert!(
                self.data.get(id).unwrap().key == k,
                "Columnar key mismatch for ClassId"
            );
        } else {
            self.stake(id, k);
        }
        self.data.get_mut(id).unwrap()
    }
}

impl<T, K: Default + PartialEq> Class<T, K> {
    pub(crate) fn get_col_or_insert(&mut self, id: ClassId) -> &mut Columnar<T, K> {
        self.get_col_or_insert_with_key(id, K::default())
    }
}

//this is moving up on the list of things to address.
//data may end up being temporally distant but HIGHLY related
//maybe class will hold pages for discrete classes to semi-pack
//then the ClassId() itself can be an enum on where to lookup
#[derive(Debug)]
pub struct Columnar<T, K = ()> {
    pub(crate) key: K,
    pub(super) vec: Vec<T>,
}

impl<T, K> Columnar<T, K> {
    pub(super) fn new(key: K) -> Self {
        Self {
            key,
            vec: Vec::new(),
        }
    }

    pub(super) fn with_capacity(key: K, capacity: usize) -> Self {
        Self {
            key,
            vec: Vec::with_capacity(capacity),
        }
    }
}

//doesnt imply that <T> need be default
impl<T, K> Default for Columnar<T, K>
where
    K: Default,
{
    fn default() -> Self {
        Self::new(K::default())
    }
}

impl<T, K> std::ops::Deref for Columnar<T, K> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T, K> std::ops::DerefMut for Columnar<T, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<T, K> Index<usize> for Columnar<T, K> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T, K> IndexMut<usize> for Columnar<T, K> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<'a, T, K> IntoIterator for &'a Columnar<T, K> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter()
    }
}

impl<'a, T, K> IntoIterator for &'a mut Columnar<T, K> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.vec.iter_mut()
    }
}

impl<T, K> IntoIterator for Columnar<T, K> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}
impl<T, K> Index<ClassId> for Class<T, K> {
    type Output = Columnar<T, K>;

    fn index(&self, index: ClassId) -> &Self::Output {
        self.data.index(index)
    }
}
impl<T, K> IndexMut<ClassId> for Class<T, K> {
    fn index_mut(&mut self, index: ClassId) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}
impl<T, K> Index<&ClassRowPtr> for Class<T, K> {
    type Output = T;

    fn index(&self, index: &ClassRowPtr) -> &Self::Output {
        &self[index.class_id][index.row_idx]
    }
}
impl<T, K> IndexMut<&ClassRowPtr> for Class<T, K> {
    fn index_mut(&mut self, index: &ClassRowPtr) -> &mut Self::Output {
        &mut self[index.class_id][index.row_idx]
    }
}

