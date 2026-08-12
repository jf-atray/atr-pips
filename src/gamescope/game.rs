use std::sync::Arc;

use slotmap::SlotMap;

use crate::assets::AssetRegistry;
use crate::brushes::Brush;
use crate::gamescope::scene::{Scene, SceneAccess};
use crate::gather::impls::gather_ref;
use crate::scripting::{Scripts, Solvers};
use crate::seek::{Seek, solve_seek};
use crate::spacial::camera::Camera;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::PipId;
use crate::tables::class::Class;
use crate::tables::class_strategy::{GrowthStrategy, rarity};
use crate::tables::core::CoreAddition;
use crate::tables::domain::Domain;
use crate::tables::system::SystemAddition;
use crate::tables::tables::Tables;

pub struct Game {
    pub domain: Domain,
    pub camera: Camera,
    pub scene: SceneAccess,
    pub asset_registry: Arc<AssetRegistry>,
    pub scripts: Scripts,
    pub solvers: Solvers,
    pub player_id: Option<PipId>,
}

impl Game {
    pub fn new(scene: SceneAccess, asset_registry: Arc<AssetRegistry>) -> Self {
        let core = CoreAddition {
            xforms: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Transform>()),
            brushes: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Brush>()),
            names: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<String>()),
            motions: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Motion>()),
        };
        let system = SystemAddition {
            pip_id: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<PipId>()),
        };
        let tables = Tables::new(core, system);

        let domain = Domain {
            tables,
            heading: SlotMap::with_key(),
            ids: SlotMap::with_key(),
        };

        let mut game = Self {
            domain,
            camera: Camera::new(),
            scene,
            asset_registry,
            scripts: Scripts::new(),
            solvers: Solvers::new(),
            player_id: None,
        };
        game.camera.zoom = 0.6;

        game.load_current();
        game
    }

    fn load_current(&mut self) {
        self.scene.current.register_tables(&mut self.domain.tables);
        let registry = &*self.asset_registry;
        self.scene.current.populate(registry, &mut self.domain);
        self.player_id = self.scene.current.player();
        self.scene.current.setup(&mut self.scripts, &mut self.solvers);
    }

    fn switch_to_next(&mut self) {
        self.scene.current.teardown(&mut self.scripts, &mut self.solvers);
        self.domain.clear();
        self.scene.current.unregister_tables(&mut self.domain.tables);
        self.scripts = Scripts::new();
        self.solvers = Solvers::new();
        self.player_id = None;

        let next = self.scene.next();
        self.scene.current = next;
        self.load_current();
    }

    fn follow_player(&mut self, dt: f32) {
        let Some(player) = self.player_id else { return };
        let ids = &self.domain.ids;
        let xforms = &self.domain.tables.core.xforms;
        if let Some(xform) = gather_ref(ids, xforms, player) {
            let goal = xform.xyz.truncate();
            let seek = Seek::with_speed(goal, 8.0);
            solve_seek(&mut self.camera.pos, &seek, dt);
        }
    }

    fn maybe_switch_scene(&mut self, dt: f32) {
        if self.scene.current.is_complete(dt, &self.domain) {
            self.switch_to_next();
        }
    }

    pub fn update(&mut self, dt: f32, aspect: f32) {
        self.scripts.update_enabled(dt, &mut self.domain, &self.solvers, &self.asset_registry);
        self.solvers.update_enabled(dt, &mut self.domain, &self.scripts, &self.asset_registry);
        self.follow_player(dt);
        self.camera.update(aspect);
        self.maybe_switch_scene(dt);
    }
}
