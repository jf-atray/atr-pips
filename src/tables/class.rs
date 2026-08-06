use std::ops::{Index, IndexMut};

use atr_plex::{Duplex, duplex};
use slotmap::{SecondaryMap, SparseSecondaryMap};

use crate::tables::{ClassId, ClassRowPtr, class_strategy::{ClassRarity, GrowthStrategy}};

#[derive(Debug)]
pub struct Class<T, K = ()> {
    pub growth: GrowthStrategy,
    pub data: ClassRarity<Columnar<T, K>>,
}
impl<T, K> Class<T, K> {
    pub fn new(rarity: duplex::Thin, growth: GrowthStrategy) -> Self {
        let data = match rarity {
            Duplex::T(_t) => Duplex::T(SecondaryMap::new()),
            Duplex::K(_k) => Duplex::K(SparseSecondaryMap::new()),
        };
        Self { growth, data }
    }
    pub fn with_capacity(capacity: usize, rarity: duplex::Thin, growth: GrowthStrategy) -> Self {
        let data = match rarity {
            Duplex::T(_t) => Duplex::T(SecondaryMap::with_capacity(capacity)),
            Duplex::K(_k) => Duplex::K(SparseSecondaryMap::with_capacity(capacity)),
        };
        Self { growth, data }
    }
    pub fn get_col(&self, id: ClassId) -> Option<&Columnar<T, K>> {
        duplex!(&self.data => { get(id) } -> unwrap)
    }
    pub fn get_col_mut(&mut self, id: ClassId) -> Option<&mut Columnar<T, K>> {
        duplex!(&mut self.data => { get_mut(id) } -> unwrap)
    }

    pub unsafe fn get_col_unchecked_mut(&mut self, id: ClassId) -> &mut Columnar<T, K> {
        duplex!(&mut self.data => |x| { unsafe{ x.get_unchecked_mut(id) } } -> unwrap)
    }
    pub unsafe fn get_col_unchecked(&self, id: ClassId) -> &Columnar<T, K> {
        duplex!(&self.data => |x| { unsafe{ x.get_unchecked(id) } } -> unwrap)
    }

    pub fn len(&self) -> usize {
        duplex!(&self.data => { len() } -> unwrap)
    }

    pub fn get_row(&self, id: &ClassRowPtr) -> Option<&T> {
        let col = self.get_col(id.class_id)?;
        Some(&col.vec[id.row_idx])
    }
    pub fn get_row_mut(&mut self, id: &ClassRowPtr) -> Option<&mut T> {
        let col = self.get_col_mut(id.class_id)?;
        Some(&mut col.vec[id.row_idx])
    }

    pub unsafe fn get_row_unchecked(&self, id: &ClassRowPtr) -> Option<&T> {
        let col = self.get_col(id.class_id)?;
        let row = unsafe { col.get_unchecked(id.row_idx) };
        Some(row)
    }
    pub unsafe fn get_row_unchecked_mut(&mut self, id: &ClassRowPtr) -> Option<&mut T> {
        let col = self.get_col_mut(id.class_id)?;
        let row = unsafe { col.get_unchecked_mut(id.row_idx) };
        Some(row)
    }
}

impl<T, K: PartialEq> Class<T, K> {
    pub fn get_col_or_insert_with_key(&mut self, id: ClassId, k: K) -> &mut Columnar<T, K> {
        match &mut self.data {
            Duplex::T(m) => {
                if m.contains_key(id) {
                    debug_assert!(
                        m.get(id).unwrap().key == k,
                        "Columnar key mismatch for ClassId"
                    );
                } else {
                    m.insert(id, Columnar::new(k));
                }
                m.get_mut(id).unwrap()
            }
            Duplex::K(m) => {
                if m.contains_key(id) {
                    debug_assert!(
                        m.get(id).unwrap().key == k,
                        "Columnar key mismatch for ClassId"
                    );
                } else {
                    m.insert(id, Columnar::new(k));
                }
                m.get_mut(id).unwrap()
            }
        }
    }
}

impl<T, K: Default + PartialEq> Class<T, K> {
    pub fn get_col_or_insert(&mut self, id: ClassId) -> &mut Columnar<T, K> {
        self.get_col_or_insert_with_key(id, K::default())
    }
}

#[derive(Debug)]
pub struct Columnar<T, K = ()> {
    pub key: K,
    pub vec: Vec<T>,
}

impl<T, K> Columnar<T, K> {
    pub fn new(key: K) -> Self {
        Self { key, vec: Vec::new() }
    }

    pub fn with_capacity(key: K, capacity: usize) -> Self {
        Self { key, vec: Vec::with_capacity(capacity) }
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
        duplex!(&self.data => { index(index) } -> unwrap )
    }
}
impl<T, K> IndexMut<ClassId> for Class<T, K> {
    fn index_mut(&mut self, index: ClassId) -> &mut Self::Output {
        duplex!(&mut self.data => { index_mut(index) } -> unwrap )
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