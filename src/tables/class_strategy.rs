use atr_plex::Duplex;
use slotmap::{SecondaryMap, SparseSecondaryMap};

use crate::tables::ClassId;

#[derive(Debug)]
pub struct GrowthStrategy {
    pub row_capacity: usize,
}
impl GrowthStrategy {
    pub const fn one_kib<T>() -> Self {
        let sz = size_of::<T>();
        if sz == 0 {
            Self::new(0)
        } else {
            Self::new(1024 / sz)
        }
    }
    pub const fn half_kib<T>() -> Self {
        let sz = size_of::<T>();
        if sz == 0 {
            Self::new(0)
        } else {
            Self::new(512 / sz)
        }
    }
    pub const fn quart_kib<T>() -> Self {
        let sz = size_of::<T>();
        if sz == 0 {
            Self::new(0)
        } else {
            Self::new(256 / sz)
        }
    }
    pub const fn new(row_capacity: usize) -> Self {
        Self { row_capacity }
    }
}

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