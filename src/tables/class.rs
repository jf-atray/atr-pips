use std::ops::{Index, IndexMut};

use atr_plex::{Duplex, duplex};
use slotmap::{SecondaryMap, SparseSecondaryMap};

use crate::tables::{ClassId, ClassRowPtr, class_strategy::{ClassRarity, GrowthStrategy}};

#[derive(Debug)]
pub struct Class<T> {
    pub growth: GrowthStrategy,
    pub data: ClassRarity<T>,
}
impl<T> Class<T> {
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
    pub fn get_col(&self, id: ClassId) -> Option<&T> {
        duplex!(&self.data => { get(id) } -> unwrap)
    }
    pub fn get_col_mut(&mut self, id: ClassId) -> Option<&mut T> {
        duplex!(&mut self.data => { get_mut(id) } -> unwrap)
    }

    pub unsafe fn get_col_unchecked_mut(&mut self, id: ClassId) -> &mut T {
        duplex!(&mut self.data => |x| { unsafe{ x.get_unchecked_mut(id) } } -> unwrap)
    }
    pub unsafe fn get_col_unchecked(&self, id: ClassId) -> &T {
        duplex!(&self.data => |x| { unsafe{ x.get_unchecked(id) } } -> unwrap)
    }

    pub fn len(&self) -> usize {
        duplex!(&self.data => { len() } -> unwrap)
    }
}

impl<T: Default> Class<T> {
    pub fn get_col_or_insert(&mut self, id: ClassId) -> &mut T {
        match &mut self.data {
            Duplex::T(m) => {
                if !m.contains_key(id) {
                    m.insert(id, T::default());
                }
                m.get_mut(id).unwrap()
            }
            Duplex::K(m) => {
                if !m.contains_key(id) {
                    m.insert(id, T::default());
                }
                m.get_mut(id).unwrap()
            }
        }
    }
}

impl<T> Class<Vec<T>> {
    pub fn row_ptr_is_valid(&self, ptr: &ClassRowPtr) -> bool {
        self.get_col(ptr.class_id)
            .is_some_and(|col| ptr.row_idx < col.len())
    }

    pub fn get_row(&self, id: &ClassRowPtr) -> Option<&T> {
        let col = self.get_col(id.class_id)?;
        Some(&col[id.row_idx])
    }
    pub fn get_row_mut(&mut self, id: &ClassRowPtr) -> Option<&mut T> {
        let col = self.get_col_mut(id.class_id)?;
        Some(&mut col[id.row_idx])
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

impl<T> Index<ClassId> for Class<T> {
    type Output = T;

    fn index(&self, index: ClassId) -> &Self::Output {
        duplex!(&self.data => { index(index) } -> unwrap )
    }
}
impl<T> IndexMut<ClassId> for Class<T> {
    fn index_mut(&mut self, index: ClassId) -> &mut Self::Output {
        duplex!(&mut self.data => { index_mut(index) } -> unwrap )
    }
}
impl<T, TCol> Index<&ClassRowPtr> for Class<TCol>
where
    TCol: Index<usize, Output = T>,
{
    type Output = T;

    fn index(&self, index: &ClassRowPtr) -> &Self::Output {
        &self[index.class_id][index.row_idx]
    }
}
impl<T, TCol> IndexMut<&ClassRowPtr> for Class<TCol>
where
    TCol: IndexMut<usize, Output = T>,
{
    fn index_mut(&mut self, index: &ClassRowPtr) -> &mut Self::Output {
        &mut self[index.class_id][index.row_idx]
    }
}