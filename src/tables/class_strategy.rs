use atr_plex::Duplex;
use slotmap::{SecondaryMap, SparseSecondaryMap};

use crate::tables::ClassId;

#[derive(Debug)]
pub struct GrowthStrategy {
    pub row_capacity: usize,
}
impl GrowthStrategy {
    pub const fn one_kib<T>() -> Self {
        let sz = Self::factor_of::<1024, T>();
        Self::new(sz)
    }
    pub const fn half_kib<T>() -> Self {
        let sz = Self::factor_of::<512, T>();
        Self::new(sz)
    }
    pub const fn quart_kib<T>() -> Self {
        let sz = Self::factor_of::<256, T>();
        Self::new(sz)
    }
    pub const fn factor_of<const N: usize, T>() -> usize {
        let typ_siz = size_of::<T>();
        N.checked_div(typ_siz).unwrap_or(0)
    }
    pub const fn new(row_capacity: usize) -> Self {
        Self { row_capacity }
    }
}

//here is the sore thumb. nice in theory but grossly useless in practice
pub type ClassRarity<T> = Duplex<SecondaryMap<ClassId, T>, SparseSecondaryMap<ClassId, T>>;

pub mod rarity {
    use atr_plex::Duplex;

    pub fn common<T, K>(t: T) -> Duplex<T, K> {
        Duplex::T(t)
    }

    pub fn rare<T, K>(k: K) -> Duplex<T, K> {
        Duplex::K(k)
    }
}
