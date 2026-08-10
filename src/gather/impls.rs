
//todo, not needed until we have PipId indirection
/*pub fn gather_ref<'a, T>(class: &'a Class<Vec<T>>, ptr: &ClassRowPtr) -> Option<&'a T> {
    class.get_row(ptr)
}

pub fn gather_mut<'a, T>(class: &'a mut Class<Vec<T>>, ptr: &ClassRowPtr) -> Option<&'a mut T> {
    class.get_row_mut(ptr)
}

pub fn gather_ref_ref<'a, T, K>(t: &'a Class<Vec<T>>, k: &'a Class<Vec<K>>, ptr: &ClassRowPtr) -> Option<(&'a T, &'a K)> {
    Some((t.get_row(ptr)?, k.get_row(ptr)?))
}

pub fn gather_mut_mut<'a, T, K>(t: &'a mut Class<Vec<T>>, k: &'a mut Class<Vec<K>>, ptr: &ClassRowPtr) -> Option<(&'a mut T, &'a mut K)> {
    Some((t.get_row_mut(ptr)?, k.get_row_mut(ptr)?))
}
*/