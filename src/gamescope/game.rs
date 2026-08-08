use std::collections::HashMap;

use slotmap::SlotMap;

use crate::brushes::Brush;
use crate::gamescope::scene::SceneAccess;
use crate::spacial::camera::Camera;
use crate::spacial::transform::Transform;
use crate::tables::class::Class;
use crate::tables::class_strategy::{GrowthStrategy, rarity};
use crate::tables::core::CoreAddition;
use crate::tables::domain::Domain;
use crate::tables::scope::{Maker, Scope};
use crate::tables::tables::Tables;

pub struct Game {
    pub domain: Domain,
    pub camera: Camera,
    pub scene: SceneAccess,
}

#[derive(Clone)]
pub struct PipMaker {
    pub xform: Transform,
    pub brush: Brush,
}

impl Maker for PipMaker {
    fn make_into(self, scope: &mut Scope) {
        scope.core.xforms = Some(self.xform);
        scope.core.brushes = Some(self.brush);
    }
}

impl Game {
    pub fn new(scene: SceneAccess) -> Self {
        let tables = Tables {
            core: CoreAddition {
                xforms: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Transform>()),
                brushes: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Brush>()),
                names: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<String>()),
            },
            additions: HashMap::new(),
        };

        let domain = Domain {
            tables,
            by_width: HashMap::new(),
            heading: SlotMap::with_key(),
        };

        Self {
            domain,
            camera: Camera::new(),
            scene,
        }
    }
    pub fn load(&mut self) {
        self.scene.current.load(&mut self.domain);
    }

    pub fn update(&mut self, _dt: f32, aspect: f32) {
        self.camera.update(aspect);
    }
}
