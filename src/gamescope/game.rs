use std::collections::HashMap;

use glam::{Quat, Vec3};
use slotmap::SlotMap;

use crate::brushes::Brush;
use crate::spacial::transform::Transform;
use crate::tables::CanvasId;
use crate::tables::class::Class;
use crate::tables::class_strategy::{GrowthStrategy, rarity};
use crate::tables::core::CoreAddition;
use crate::tables::domain::Domain;
use crate::tables::scope::{Maker, Scope};
use crate::tables::tables::Tables;

pub struct Game {
    pub domain: Domain,
}

struct PipMaker {
    xform: Transform,
    brush: Brush,
}

impl Maker for PipMaker {
    fn make_into(self, scope: &mut Scope) {
        scope.core.xforms = Some(self.xform);
        scope.core.brushes = Some(self.brush);
    }
}

impl Game {
    pub fn new(canvas_id: CanvasId) -> Self {
        let tables = Tables {
            core: CoreAddition {
                xforms: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Transform>()),
                brushes: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Brush>()),
                names: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<String>()),
            },
            additions: HashMap::new(),
        };

        let mut domain = Domain {
            tables,
            by_width: HashMap::new(),
            heading: SlotMap::with_key(),
        };

        let maker = PipMaker {
            xform: Transform {
                xyz: Vec3::new(-0.5, 0.0, 0.0),
                rot: Quat::IDENTITY,
            },
            brush: Brush { canvas: canvas_id },
        };
        domain.make(maker);

        let maker = PipMaker {
            xform: Transform {
                xyz: Vec3::new(0.5, 0.1, 0.0),
                rot: Quat::IDENTITY,
            },
            brush: Brush { canvas: canvas_id },
        };
        domain.make(maker);

        Self { domain }
    }

    pub fn update(&mut self, _dt: f32) {}
}
