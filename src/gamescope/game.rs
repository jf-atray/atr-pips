use std::collections::HashMap;

use slotmap::SlotMap;

use crate::assets::AssetRegistry;
use crate::brushes::Brush;
use crate::gamescope::motion::MotionSolver;
use crate::gamescope::scene::SceneAccess;
use crate::spacial::camera::Camera;
use crate::spacial::motion::Motion;
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
    pub asset_registry: AssetRegistry,
    pub motion_solver: MotionSolver,
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
        scope.core.motions = Some(Motion::random_unit(),);
    }
}

impl Game {
    pub fn new(scene: SceneAccess, asset_registry: AssetRegistry) -> Self {
        let tables = Tables {
            core: CoreAddition {
                xforms: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Transform>()),
                brushes: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Brush>()),
                names: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<String>()),
                motions: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Motion>()),
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
            asset_registry,
            motion_solver: MotionSolver::new(),
        }
    }
    pub fn load(&mut self) {
        let registry = &self.asset_registry;
        self.scene.current.load(registry, &mut self.domain);
    }

    //this is where we'll need to handle additions, scripts, solvers, etc.
    pub fn update(&mut self, dt: f32, aspect: f32) {
        self.motion_solver.update(&mut self.domain.tables, dt);
        self.camera.update(aspect);
    }
}
