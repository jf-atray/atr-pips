use atr_plex::Duplex;
use atr_plex::duplex;
use slotmap::{secondary, sparse_secondary};

use crate::tables::ClassId;
use crate::tables::class::{Class, Columnar};
use crate::tables::class_strategy::ClassRarity;

type ClassIterRef<'a, T, K = ()> = Duplex<
    secondary::Iter<'a, ClassId, Columnar<T, K>>,
    sparse_secondary::Iter<'a, ClassId, Columnar<T, K>>,
>;
pub struct QueryRefRefIter<'a, T: 'a, K: 'a, TKey: 'a, KKey: 'a> {
    smallest_source: ClassIterRef<'a, T, TKey>,
    k_source: ClassRarity<Columnar<K, KKey>>::Ref<'a>,
    t_key: &'a TKey,
    k_key: &'a KKey,
}

impl<'a, T, K, TKey, KKey> Iterator for QueryRefRefIter<'a, T, K, TKey, KKey>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    type Item = (&'a Columnar<T, TKey>, &'a Columnar<K, KKey>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((class_id, t)) = duplex!(&mut self.smallest_source => { next() } -> unwrap) {
            if let Some(k) = duplex!(&self.k_source => { get(class_id) } -> unwrap)
                && &t.key == self.t_key && &k.key == self.k_key {
                    return Some((t, k));
                }
        }
        None
    }
}


type ClassIterMut<'a, T, K = ()> = Duplex<
    secondary::IterMut<'a, ClassId, Columnar<T, K>>,
    sparse_secondary::IterMut<'a, ClassId, Columnar<T, K>>,
>;

pub struct QueryMutMutIter<'a, T: 'a, K: 'a, TKey: 'a, KKey: 'a> {
    smallest_source: ClassIterMut<'a, T, TKey>,
    k_source: ClassRarity<Columnar<K, KKey>>::Mut<'a>,
    t_key: &'a TKey,
    k_key: &'a KKey,
}

impl<'a, T, K, TKey, KKey> Iterator for QueryMutMutIter<'a, T, K, TKey, KKey>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    type Item = (&'a mut Columnar<T, TKey>, &'a mut Columnar<K, KKey>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((class_id, t)) = duplex!(&mut self.smallest_source => { next() } -> unwrap) {
            if let Some(k) = duplex!(&mut self.k_source => { get_mut(class_id) } -> unwrap)
                && &t.key == self.t_key && &k.key == self.k_key {
                    let k = unsafe { &mut *std::ptr::from_mut::<Columnar<K, KKey>>(k) };
                    return Some((t, k));
                }
        }
        None
    }
}

pub fn query_ref<'a, T, K>(class: &'a Class<T, K>) -> impl Iterator<Item = &'a Columnar<T, K>>
where
    T: 'a,
    K: 'a,
{
    let mut columns = duplex!(&class.data => values());
    std::iter::from_fn(move || duplex!(&mut columns => { next() } -> unwrap))
}

pub fn query_mut<'a, T, K>(
    class: &'a mut Class<T, K>,
) -> impl Iterator<Item = &'a mut Columnar<T, K>>
where
    T: 'a,
    K: 'a,
{
    let mut columns = duplex!(&mut class.data => values_mut());
    std::iter::from_fn(move || duplex!(&mut columns => { next() } -> unwrap))
}

pub fn query_ref_ref<'a, T, K, TKey, KKey>(
    t: &'a Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a Class<K, KKey>,
    k_key: &'a KKey,
) -> impl Iterator<Item = (&'a Columnar<T, TKey>, &'a Columnar<K, KKey>)>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    let t_len = duplex!(&t.data => { len() } -> unwrap);
    let k_len = duplex!(&k.data => { len() } -> unwrap);
    let mut duplex = if t_len <= k_len {
        let smallest_source = duplex!(&t.data => iter());
        let k_source = k.data.as_ref();
        Duplex::T(QueryRefRefIter {
            smallest_source,
            k_source,
            t_key,
            k_key,
        })
    } else {
        let smallest_source = duplex!(&k.data => iter());
        let k_source = t.data.as_ref();
        Duplex::K(QueryRefRefIter {
            smallest_source,
            k_source,
            t_key: k_key,
            k_key: t_key,
        })
    };
    std::iter::from_fn(move || match &mut duplex {
        Duplex::T(t) => t.next(),
        Duplex::K(k) => k.next().map(|(k, t)| (t, k)),
    })
}

pub fn query_mut_mut<'a, T, K, TKey, KKey>(
    t: &'a mut Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a mut Class<K, KKey>,
    k_key: &'a KKey,
) -> impl Iterator<Item = (&'a mut Columnar<T, TKey>, &'a mut Columnar<K, KKey>)>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    let t_len = duplex!(&t.data => { len() } -> unwrap);
    let k_len = duplex!(&k.data => { len() } -> unwrap);
    let mut duplex = if t_len <= k_len {
        let smallest_source = duplex!(&mut t.data => iter_mut());
        let k_source = k.data.as_mut();
        Duplex::T(QueryMutMutIter {
            smallest_source,
            k_source,
            t_key,
            k_key,
        })
    } else {
        let smallest_source = duplex!(&mut k.data => iter_mut());
        let k_source = t.data.as_mut();
        Duplex::K(QueryMutMutIter {
            smallest_source,
            k_source,
            t_key: k_key,
            k_key: t_key,
        })
    };
    std::iter::from_fn(move || match &mut duplex {
        Duplex::T(t) => t.next(),
        Duplex::K(k) => k.next().map(|(k, t)| (t, k)),
    })
}
