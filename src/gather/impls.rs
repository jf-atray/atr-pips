use slotmap::SlotMap;

use crate::tables::{ClassRowPtr, PipId, class::Class};

pub fn gather_ref<'a, T, K>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    class: &'a Class<T, K>,
    pip: PipId,
) -> Option<&'a T> {
    let ptr = ids.get(pip)?;
    class.get_row(ptr)
}

pub fn gather_mut<'a, T, K>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    class: &'a mut Class<T, K>,
    pip: PipId,
) -> Option<&'a mut T> {
    let ptr = ids.get(pip)?;
    class.get_row_mut(ptr)
}

pub fn gather_pair_ref<'a, T, K, U, L>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    a: &'a Class<T, K>,
    b: &'a Class<U, L>,
    pip: PipId,
) -> Option<(&'a T, &'a U)> {
    let ptr = ids.get(pip)?;
    Some((a.get_row(ptr)?, b.get_row(ptr)?))
}

pub fn gather_pair_mut<'a, T, K, U, L>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    a: &'a mut Class<T, K>,
    b: &'a mut Class<U, L>,
    pip: PipId,
) -> Option<(&'a mut T, &'a mut U)> {
    let ptr = ids.get(pip)?;
    Some((a.get_row_mut(ptr)?, b.get_row_mut(ptr)?))
}
