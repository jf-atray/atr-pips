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

#[cfg(test)]
mod tests {
    use glam::{Quat, Vec3};
    use slotmap::SlotMap;
    use std::collections::HashMap;

    use super::*;
    use crate::brushes::Brush;
    use crate::spacial::motion::Motion;
    use crate::spacial::transform::Transform;
    use crate::tables::PipId;
    use crate::tables::class::Class;
    use crate::tables::class_strategy::{GrowthStrategy, rarity};
    use crate::tables::core::CoreAddition;
    use crate::tables::domain::Domain;
    use crate::tables::scope::{Maker, Scope};
    use crate::tables::system::SystemAddition;
    use crate::tables::tables::Tables;

    struct M(Transform, Motion);

    impl Maker for M {
        fn make_into(self, scope: &mut Scope) {
            scope.core.xforms = Some(self.0);
            scope.core.motions = Some(self.1);
        }
    }

    fn tables() -> Tables {
        Tables {
            core: CoreAddition {
                xforms: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Transform>()),
                brushes: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Brush>()),
                names: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<String>()),
                motions: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Motion>()),
            },
            additions: HashMap::new(),
            system: SystemAddition {
                pip_id: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<PipId>()),
            },
        }
    }

    #[test]
    fn gather_round_trip() {
        let mut domain = Domain::new(tables());
        let id = domain.make(M(Transform { xyz: Vec3::ONE, rot: Quat::IDENTITY }, Motion { vel: Vec3::ZERO }));

        let xform = gather_ref(&domain.ids, &domain.tables.core.xforms, id).unwrap();
        assert_eq!(xform.xyz, Vec3::ONE);

        let xform = gather_mut(&domain.ids, &mut domain.tables.core.xforms, id).unwrap();
        xform.xyz = Vec3::new(2.0, 2.0, 2.0);

        let xform = gather_ref(&domain.ids, &domain.tables.core.xforms, id).unwrap();
        assert_eq!(xform.xyz, Vec3::new(2.0, 2.0, 2.0));
    }
}
