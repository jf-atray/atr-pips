use slotmap::SlotMap;

use crate::ecs::{ClassRowPtr, PipId, class::Class};

pub(crate) fn gather_ref<'a, T, K>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    class: &'a Class<T, K>,
    pip: PipId,
) -> Option<&'a T> {
    let ptr = ids.get(pip)?;
    class.get_row(ptr)
}

pub(crate) fn gather_mut<'a, T, K>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    class: &'a mut Class<T, K>,
    pip: PipId,
) -> Option<&'a mut T> {
    let ptr = ids.get(pip)?;
    class.get_row_mut(ptr)
}

pub(crate) fn gather_pair_ref<'a, T, K, U, L>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    a: &'a Class<T, K>,
    b: &'a Class<U, L>,
    pip: PipId,
) -> Option<(&'a T, &'a U)> {
    let ptr = ids.get(pip)?;
    Some((a.get_row(ptr)?, b.get_row(ptr)?))
}

pub(crate) fn gather_pair_mut<'a, T, K, U, L>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    a: &'a mut Class<T, K>,
    b: &'a mut Class<U, L>,
    pip: PipId,
) -> Option<(&'a mut T, &'a mut U)> {
    let ptr = ids.get(pip)?;
    Some((a.get_row_mut(ptr)?, b.get_row_mut(ptr)?))
}

pub(crate) fn gather_two_mut<'a, T, K>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    class: &'a mut Class<T, K>,
    pip_a: PipId,
    pip_b: PipId,
) -> Option<(&'a mut T, &'a mut T)> {
    let ptr_a = ids.get(pip_a)?;
    let ptr_b = ids.get(pip_b)?;
    class.get_two_rows_mut(ptr_a, ptr_b)
}

pub(crate) fn gather_two_pair_mut<'a, T, K, U, L>(
    ids: &SlotMap<PipId, ClassRowPtr>,
    a: &'a mut Class<T, K>,
    b: &'a mut Class<U, L>,
    pip_a: PipId,
    pip_b: PipId,
) -> Option<(&'a mut T, &'a mut U, &'a mut T, &'a mut U)> {
    let ptr_a = ids.get(pip_a)?;
    let ptr_b = ids.get(pip_b)?;
    let (a1, a2) = a.get_two_rows_mut(ptr_a, ptr_b)?;
    let (b1, b2) = b.get_two_rows_mut(ptr_a, ptr_b)?;
    Some((a1, b1, a2, b2))
}
