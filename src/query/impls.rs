use crate::ecs::class::{Class, Columnar};

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
    let (t_data, t_class) = (&t.data, &t.class);
    let (k_data, k_class) = (&k.data, &k.class);
    let smallest = if t_class.len() <= k_class.len() {
        t_class
    } else {
        k_class
    };

    let coroutine = std::iter::iter!(move || {
        for class_id in smallest {
            if let Some(t_col) = t_data.get(*class_id)
                && let Some(k_col) = k_data.get(*class_id)
                && &t_col.key == t_key
                && &k_col.key == k_key
            {
                yield (t_col, k_col);
            }
        }
    });
    coroutine()
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
    let (t_data, t_class) = (&mut t.data, &t.class);
    let (k_data, k_class) = (&mut k.data, &k.class);
    let smallest = if t_class.len() <= k_class.len() {
        t_class
    } else {
        k_class
    };

    let coroutine = std::iter::iter!(move || {
        for class_id in smallest {
            if let Some(t_col) = t_data.get_mut(*class_id)
                && let Some(k_col) = k_data.get_mut(*class_id)
                && &t_col.key == t_key
                && &k_col.key == k_key
            {
                let t_col = unsafe { &mut *std::ptr::from_mut(t_col) };
                let k_col = unsafe { &mut *std::ptr::from_mut(k_col) };
                yield (t_col, k_col);
            }
        }
    });
    coroutine()
}

pub fn query_mut_ref<'a, T, K, TKey, KKey>(
    t: &'a mut Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a Class<K, KKey>,
    k_key: &'a KKey,
) -> impl Iterator<Item = (&'a mut Columnar<T, TKey>, &'a Columnar<K, KKey>)>
where
    T: 'a,
    K: 'a,
    TKey: 'a + PartialEq,
    KKey: 'a + PartialEq,
{
    let (t_data, t_class) = (&mut t.data, &t.class);
    let (k_data, k_class) = (&k.data, &k.class);
    let smallest = if t_class.len() <= k_class.len() {
        t_class
    } else {
        k_class
    };

    let coroutine = std::iter::iter!(move || {
        for class_id in smallest {
            if let Some(t_col) = t_data.get_mut(*class_id)
                && let Some(k_col) = k_data.get(*class_id)
                && &t_col.key == t_key
                && &k_col.key == k_key
            {
                let t_col = unsafe { &mut *std::ptr::from_mut(t_col) };
                yield (t_col, k_col);
            }
        }
    });
    coroutine()
}

pub fn query_mut_mut_mut<'a, T, K, L, TKey, KKey, LKey>(
    t: &'a mut Class<T, TKey>,
    t_key: &'a TKey,
    k: &'a mut Class<K, KKey>,
    k_key: &'a KKey,
    l: &'a mut Class<L, LKey>,
    l_key: &'a LKey,
) -> impl Iterator<
    Item = (
        &'a mut Columnar<T, TKey>,
        &'a mut Columnar<K, KKey>,
        &'a mut Columnar<L, LKey>,
    ),
>
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
        t_class
    } else if k_class.len() <= t_class.len() && k_class.len() <= l_class.len() {
        k_class
    } else {
        l_class
    };

    let coroutine = std::iter::iter!(move || {
        for class_id in smallest {
            if let Some(t_col) = t_data.get_mut(*class_id)
                && let Some(k_col) = k_data.get_mut(*class_id)
                && let Some(l_col) = l_data.get_mut(*class_id)
                && &t_col.key == t_key
                && &k_col.key == k_key
                && &l_col.key == l_key
            {
                let t_col = unsafe { &mut *std::ptr::from_mut(t_col) };
                let k_col = unsafe { &mut *std::ptr::from_mut(k_col) };
                let l_col = unsafe { &mut *std::ptr::from_mut(l_col) };
                yield (t_col, k_col, l_col);
            }
        }
    });
    coroutine()
}
