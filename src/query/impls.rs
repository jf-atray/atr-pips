use atr_plex::Duplex;
use atr_plex::duplex;
use slotmap::{secondary, sparse_secondary};

use crate::tables::class_strategy::ClassRarity;
use crate::tables::{ClassId, class::Class};


type ClassIterRef<'a, T> = Duplex<
    secondary::Iter<'a, ClassId, Vec<T>>,
    sparse_secondary::Iter<'a, ClassId, Vec<T>>,
>;
pub struct QueryRefRefIter<'a, T: 'a, K: 'a> {
    smallest_source: ClassIterRef<'a, T>,
    k_source: ClassRarity<Vec<K>>::Ref<'a>,
}

impl<'a, T: 'a, K: 'a> Iterator for QueryRefRefIter<'a, T, K> {
    type Item = (&'a Vec<T>, &'a Vec<K>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((class_id, t)) = duplex!(&mut self.smallest_source => { next() } -> unwrap) {
            if let Some(k) = duplex!(&self.k_source => { get(class_id) } -> unwrap) {
                return Some((t, k));
            }
        }
        None
    }
}

type ClassIterMut<'a, T> = Duplex<
    secondary::IterMut<'a, ClassId, Vec<T>>,
    sparse_secondary::IterMut<'a, ClassId, Vec<T>>,
>;

pub struct QueryMutMutIter<'a, T: 'a, K: 'a> {
    smallest_source: ClassIterMut<'a, T>,
    k_source: ClassRarity<Vec<K>>::Mut<'a>,
}

impl<'a, T: 'a, K: 'a> Iterator for QueryMutMutIter<'a, T, K> {
    type Item = (&'a mut Vec<T>, &'a mut Vec<K>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((class_id, t)) = duplex!(&mut self.smallest_source => { next() } -> unwrap) {
            if let Some(k) = duplex!(&mut self.k_source => { get_mut(class_id) } -> unwrap) {
                let k = unsafe { &mut *(k as *mut Vec<K>) };
                return Some((t, k));
            }
        }
        None
    }
}

pub fn query_ref<'a, T: 'a>(class: &'a Class<Vec<T>>) -> impl Iterator<Item = &'a Vec<T>> {
    let mut columns = duplex!(&class.data => iter());
    std::iter::from_fn(move || {
        duplex!(&mut columns => { next() } -> unwrap).map(|(_, col)| col)
    })
}

pub fn query_mut<'a, T: 'a>(class: &'a mut Class<Vec<T>>) -> impl Iterator<Item = &'a mut Vec<T>> {
    let mut columns = duplex!(&mut class.data => iter_mut());
    std::iter::from_fn(move || {
        duplex!(&mut columns => { next() } -> unwrap).map(|(_, col)| col)
    })
}

pub fn query_ref_ref<'a, T, K>(
    t: &'a Class<Vec<T>>,
    k: &'a Class<Vec<K>>,
) -> Duplex<QueryRefRefIter<'a, T, K>, QueryRefRefIter<'a, K, T>>
where
    T: 'a,
    K: 'a,
{
    let t_len = duplex!(&t.data => { len() } -> unwrap);
    let k_len = duplex!(&k.data => { len() } -> unwrap);
    if t_len <= k_len {
        let smallest_source = duplex!(&t.data => iter());
        let k_source = k.data.as_ref();
        Duplex::T(QueryRefRefIter {
            smallest_source,
            k_source,
        })
    } else {
        let smallest_source = duplex!(&k.data => iter());
        let k_source = t.data.as_ref();
        Duplex::K(QueryRefRefIter {
            smallest_source,
            k_source,
        })
    }
}

pub fn query_mut_mut<'a, T, K>(
    t: &'a mut Class<Vec<T>>,
    k: &'a mut Class<Vec<K>>,
) -> Duplex<QueryMutMutIter<'a, T, K>, QueryMutMutIter<'a, K, T>>
where
    T: 'a,
    K: 'a,
{
    let t_len = duplex!(&t.data => { len() } -> unwrap);
    let k_len = duplex!(&k.data => { len() } -> unwrap);
    if t_len <= k_len {
        let smallest_source = duplex!(&mut t.data => iter_mut());
        let k_source = k.data.as_mut();
        Duplex::T(QueryMutMutIter {
            smallest_source,
            k_source,
        })
    } else {
        let smallest_source = duplex!(&mut k.data => iter_mut());
        let k_source = t.data.as_mut();
        Duplex::K(QueryMutMutIter {
            smallest_source,
            k_source,
        })
    }
}