use slotmap::SecondaryMap;

use crate::tables::ClassId;
use crate::tables::class::{Class, Columnar};

pub fn call1<A, F: FnMut(A)>(mut f: F, a: A) {
    f(a)
}

pub fn call2<A, B, F: FnMut(A, B)>(mut f: F, a: A, b: B) {
    f(a, b)
}

pub fn call3<A, B, C, F: FnMut(A, B, C)>(mut f: F, a: A, b: B, c: C) {
    f(a, b, c)
}

pub fn query_ref<'a, T, K>(
    class: &'a Class<T, K>,
    key: K,
) -> impl Iterator<Item = &'a Columnar<T, K>>
where
    T: 'a,
    K: 'a + PartialEq,
{
    class.data.values().filter(move |v| v.key == key)
}

pub fn query_mut<'a, T, K>(
    class: &'a mut Class<T, K>,
    key: K,
) -> impl Iterator<Item = &'a mut Columnar<T, K>>
where
    T: 'a,
    K: 'a + PartialEq,
{
    class.data.values_mut().filter(move |v| v.key == key)
}

pub struct QueryRefRefIter<'a, T, K, TKey, KKey> {
    smallest: std::collections::btree_set::Iter<'a, ClassId>,
    t_data: &'a SecondaryMap<ClassId, Columnar<T, TKey>>,
    k_data: &'a SecondaryMap<ClassId, Columnar<K, KKey>>,
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
        for class_id in &mut self.smallest {
            if let Some(t_col) = self.t_data.get(*class_id)
                && let Some(k_col) = self.k_data.get(*class_id)
                && &t_col.key == self.t_key
                && &k_col.key == self.k_key
            {
                return Some((t_col, k_col));
            }
        }
        None
    }
}

pub fn query_ref_ref<'a, T, K, TKey, KKey>(
    t: &'a Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a Class<K, KKey>,
    k_key: &'a KKey,
) -> QueryRefRefIter<'a, T, K, TKey, KKey>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    let (t_data, t_class) = (&t.data, &t.class);
    let (k_data, k_class) = (&k.data, &k.class);
    let smallest = if t_class.len() <= k_class.len() {
        t_class.iter()
    } else {
        k_class.iter()
    };
    QueryRefRefIter {
        smallest,
        t_data,
        k_data,
        t_key,
        k_key,
    }
}

pub struct QueryMutMutIter<'a, T, K, TKey, KKey> {
    smallest: std::collections::btree_set::Iter<'a, ClassId>,
    t_data: &'a mut SecondaryMap<ClassId, Columnar<T, TKey>>,
    k_data: &'a mut SecondaryMap<ClassId, Columnar<K, KKey>>,
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
        for class_id in &mut self.smallest {
            if let Some(t_col) = self.t_data.get_mut(*class_id)
                && let Some(k_col) = self.k_data.get_mut(*class_id)
                && &t_col.key == self.t_key
                && &k_col.key == self.k_key
            {
                let t = unsafe { &mut *std::ptr::from_mut::<Columnar<T, TKey>>(t_col) };
                let k = unsafe { &mut *std::ptr::from_mut::<Columnar<K, KKey>>(k_col) };
                return Some((t, k));
            }
        }
        None
    }
}

pub fn query_mut_mut<'a, T, K, TKey, KKey>(
    t: &'a mut Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a mut Class<K, KKey>,
    k_key: &'a KKey,
) -> QueryMutMutIter<'a, T, K, TKey, KKey>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    let (t_data, t_class) = (&mut t.data, &t.class);
    let (k_data, k_class) = (&mut k.data, &k.class);
    let smallest = if t_class.len() <= k_class.len() {
        t_class.iter()
    } else {
        k_class.iter()
    };
    QueryMutMutIter {
        smallest,
        t_data,
        k_data,
        t_key,
        k_key,
    }
}

pub struct QueryMutRefIter<'a, T, K, TKey, KKey> {
    smallest: std::collections::btree_set::Iter<'a, ClassId>,
    t_data: &'a mut SecondaryMap<ClassId, Columnar<T, TKey>>,
    k_data: &'a SecondaryMap<ClassId, Columnar<K, KKey>>,
    t_key: &'a TKey,
    k_key: &'a KKey,
}

impl<'a, T, K, TKey, KKey> Iterator for QueryMutRefIter<'a, T, K, TKey, KKey>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    type Item = (&'a mut Columnar<T, TKey>, &'a Columnar<K, KKey>);

    fn next(&mut self) -> Option<Self::Item> {
        for class_id in &mut self.smallest {
            if let Some(t_col) = self.t_data.get_mut(*class_id)
                && let Some(k_col) = self.k_data.get(*class_id)
                && &t_col.key == self.t_key
                && &k_col.key == self.k_key
            {
                // SAFETY: `smallest` yields unique `class_id`s, so each `t_col` is disjoint.
                let t = unsafe { &mut *std::ptr::from_mut::<Columnar<T, TKey>>(t_col) };
                return Some((t, k_col));
            }
        }
        None
    }
}

pub fn query_mut_ref<'a, T, K, TKey, KKey>(
    t: &'a mut Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a Class<K, KKey>,
    k_key: &'a KKey,
) -> QueryMutRefIter<'a, T, K, TKey, KKey>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    let (t_data, t_class) = (&mut t.data, &t.class);
    let (k_data, k_class) = (&k.data, &k.class);
    let smallest = if t_class.len() <= k_class.len() {
        t_class.iter()
    } else {
        k_class.iter()
    };
    QueryMutRefIter {
        smallest,
        t_data,
        k_data,
        t_key,
        k_key,
    }
}

pub struct QueryMutMutMutIter<'a, T, K, L, TKey, KKey, LKey> {
    smallest: std::collections::btree_set::Iter<'a, ClassId>,
    t_data: &'a mut SecondaryMap<ClassId, Columnar<T, TKey>>,
    k_data: &'a mut SecondaryMap<ClassId, Columnar<K, KKey>>,
    l_data: &'a mut SecondaryMap<ClassId, Columnar<L, LKey>>,
    t_key: &'a TKey,
    k_key: &'a KKey,
    l_key: &'a LKey,
}

impl<'a, T, K, L, TKey, KKey, LKey> Iterator for QueryMutMutMutIter<'a, T, K, L, TKey, KKey, LKey>
where
    T: 'a,
    K: 'a,
    L: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
    LKey: 'a + PartialEq,
{
    type Item = (
        &'a mut Columnar<T, TKey>,
        &'a mut Columnar<K, KKey>,
        &'a mut Columnar<L, LKey>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        for class_id in &mut self.smallest {
            if let Some(t_col) = self.t_data.get_mut(*class_id)
                && let Some(k_col) = self.k_data.get_mut(*class_id)
                && let Some(l_col) = self.l_data.get_mut(*class_id)
                && &t_col.key == self.t_key
                && &k_col.key == self.k_key
                && &l_col.key == self.l_key
            {
                // SAFETY: unique `class_id`s from a BTreeSet; three disjoint maps.
                let t = unsafe { &mut *std::ptr::from_mut::<Columnar<T, TKey>>(t_col) };
                let k = unsafe { &mut *std::ptr::from_mut::<Columnar<K, KKey>>(k_col) };
                let l = unsafe { &mut *std::ptr::from_mut::<Columnar<L, LKey>>(l_col) };
                return Some((t, k, l));
            }
        }
        None
    }
}

pub fn query_mut_mut_mut<'a, T, K, L, TKey, KKey, LKey>(
    t: &'a mut Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a mut Class<K, KKey>,
    k_key: &'a KKey,
    l: &'a mut Class<L, LKey>,
    l_key: &'a LKey,
) -> QueryMutMutMutIter<'a, T, K, L, TKey, KKey, LKey>
where
    T: 'a,
    K: 'a,
    L: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
    LKey: 'a + PartialEq,
{
    let (t_data, t_class) = (&mut t.data, &t.class);
    let (k_data, k_class) = (&mut k.data, &k.class);
    let (l_data, l_class) = (&mut l.data, &l.class);
    let smallest = if t_class.len() <= k_class.len() && t_class.len() <= l_class.len() {
        t_class.iter()
    } else if k_class.len() <= t_class.len() && k_class.len() <= l_class.len() {
        k_class.iter()
    } else {
        l_class.iter()
    };
    QueryMutMutMutIter {
        smallest,
        t_data,
        k_data,
        l_data,
        t_key,
        k_key,
        l_key,
    }
}
