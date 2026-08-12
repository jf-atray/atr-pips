use std::any::Any;

use slotmap::SlotMap;

use crate::tables::{ClassId, ClassRowPtr, PipId, scope::{Scope, Maker}, tables::Tables, partition::Addition};

pub struct Domain {
    pub tables: Tables,
    pub heading: SlotMap<ClassId, usize>,
    pub ids: SlotMap<PipId, ClassRowPtr>,
}

impl Domain {
    pub fn new(tables: Tables) -> Self {
        Self {
            tables,
            heading: SlotMap::with_key(),
            ids: SlotMap::with_key(),
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.heading.clear();
        self.tables.clear();
    }

    pub fn make<M: Maker>(&mut self, maker: M) -> PipId {
        //acquires a generational index
        let pip = self.ids.insert(ClassRowPtr::new(ClassId::default(), 0));
        let ptr = self.commit(pip, maker);
        //backfill with what class we actually discovered
        self.ids[pip] = ptr;
        pip
    }

    pub fn destroy(&mut self, pip: PipId) {
        let Some(ptr) = self.ids.get(pip).cloned() else {
            return;
        };

        self.destroy_ptr(&ptr);
        let displaced = self.tables.system.pip_id
            .get_col(ptr.class_id)
            .and_then(|col| col.get(ptr.row_idx))
            .copied();

        if let Some(displaced) = displaced
            && let Some(entry) = self.ids.get_mut(displaced) {
                *entry = ClassRowPtr::new(ptr.class_id, ptr.row_idx);
            }

        self.ids.remove(pip);
    }



    fn commit<M: Maker>(&mut self, pip: PipId, maker: M) -> ClassRowPtr {
        let mut scope = Scope::default();

        for (addition_id, addition) in self.tables.addition_entries() {
            let view = addition.view_default();
            let view_id = view.as_any_ref().type_id();
            scope.additions.insert(view_id, (*addition_id, view));
        }

        maker.make_into(&mut scope);
        scope.system.pip_id = Some(pip);

        let width = scope.width();


        let mut class_id = None;
        //todo in youfirst I actually itered the smallest commited class
        //but depending on the kind of slotmap this might actually be
        //less than useful.
        //doublecheck secondarymap because if sparce is still o(1) for a little more memory
        //and gains a fwd walk key iter... yummy
        for id in self.heading.iter().filter(|(_,v)| **v == width ).map(|(k,_)| k){
            if scope.matches(id, &self.tables) {
                class_id = Some(id);
                break;
            }
        }

        let class_id = class_id.unwrap_or_else(|| {
            self.heading.insert(width)
        });

        let row_idx = scope.commit(class_id, &mut self.tables).unwrap();
        ClassRowPtr::new(class_id, row_idx)
    }

    fn destroy_ptr(&mut self, ptr: &ClassRowPtr) {
        self.tables.core.destroy(ptr.class_id, ptr.row_idx);
        self.tables.system.destroy(ptr.class_id, ptr.row_idx);

        for addition in self.tables.addition_values_mut() {
            addition.destroy(ptr.class_id, ptr.row_idx);
        }
    }
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
    fn clear_clears_all() {
        let mut domain = Domain::new(tables());
        let _ = domain.make(M(Transform { xyz: Vec3::ONE, rot: Quat::IDENTITY }, Motion { vel: Vec3::ZERO }));
        let _ = domain.make(M(Transform { xyz: Vec3::ONE, rot: Quat::IDENTITY }, Motion { vel: Vec3::ZERO }));
        assert_eq!(domain.ids.len(), 2);
        assert!(domain.tables.core.xforms.len() > 0);
        domain.clear();
        assert_eq!(domain.ids.len(), 0);
        assert_eq!(domain.heading.len(), 0);
        assert_eq!(domain.tables.core.xforms.len(), 0);
    }

    #[test]
    fn vec_flush_pattern() {
        let mut domain = Domain::new(tables());
        let mut to_spawn = Vec::new();
        to_spawn.push(M(Transform { xyz: Vec3::ONE, rot: Quat::IDENTITY }, Motion { vel: Vec3::ZERO }));
        to_spawn.push(M(Transform { xyz: Vec3::ONE, rot: Quat::IDENTITY }, Motion { vel: Vec3::ZERO }));
        let mut ids = Vec::new();
        for m in to_spawn {
            ids.push(domain.make(m));
        }
        let mut to_destroy = Vec::new();
        for id in &ids {
            to_destroy.push(*id);
        }
        for id in to_destroy {
            domain.destroy(id);
        }
        assert_eq!(domain.ids.len(), 0);
    }
}