use glam::{Vec2, Vec3, Vec4};
use rand::Rng;

use crate::arena::makers::{ActorBlueprint, PickupBlueprint, SpawnerBlueprint};
use crate::arena::scripts::PilotScript;
use crate::arena::solvers::{BoundsSolver, MovementSolver, PickupSolver, ProjectileSolver, SpawnerSolver};
use crate::arena::tables::{
    HealthAddition, HealthData, HealthPickupAddition, HealthPickupData, PilotAddition, PilotData,
    PilotState, SpawnerData, SpawnerAddition, Team, TeamAddition, ProjectileAddition,
};
use crate::assets::AssetRegistry;
use crate::brushes::Brush;
use crate::gamescope::scene::Scene;
use crate::scripting::{EveryScript, ScriptHost, Scripts, Solvers};
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::class::Class;
use crate::tables::class_strategy::{rarity, GrowthStrategy};
use crate::tables::domain::Domain;
use crate::tables::tables::Tables;
use crate::tables::PipId;

const SPAWN_BOUNDS: f32 = 12.0;

fn make_brush(registry: &AssetRegistry, name: &str, scale: f32) -> Brush {
    let e = registry.get(name);
    Brush {
        canvas: e.canvas,
        material: e.material,
        scale: e.natural_scale * scale,
        color: Vec4::ONE,
    }
}

fn random_pos(rng: &mut impl Rng) -> Vec2 {
    Vec2::new(
        rng.random::<f32>() * SPAWN_BOUNDS * 2.0 - SPAWN_BOUNDS,
        rng.random::<f32>() * SPAWN_BOUNDS * 2.0 - SPAWN_BOUNDS,
    )
}

fn register_common(tables: &mut Tables) {
    if tables.get::<TeamAddition>().is_none() {
        tables.add(TeamAddition { team: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<Team>()) });
    }
    if tables.get::<PilotAddition>().is_none() {
        tables.add(PilotAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<PilotData>()) });
    }
}

fn unregister_common(tables: &mut Tables) {
    let _ = tables.remove::<TeamAddition>();
    let _ = tables.remove::<PilotAddition>();
}

fn set_enemies_to_chase(domain: &mut Domain, player: PipId) {
    let mut additions = crate::tables::tables::TablesAdditions::new(&mut domain.tables.additions);
    let (pilot, team) = additions.get_many_mut::<PilotAddition, TeamAddition>().unwrap();

    for (class_id, team_col) in team.team.columns() {
        let Some(pilot_col) = pilot.data.get_col_mut(class_id) else { continue };
        for i in 0..team_col.len().min(pilot_col.len()) {
            if team_col[i] == Team::Enemy {
                pilot_col[i].state = PilotState::Chase { target: player };
            }
        }
    }
}

pub struct SplashScene {
    timer: f32,
    player: Option<PipId>,
}

impl SplashScene {
    pub fn new() -> Self {
        Self { timer: 0.0, player: None }
    }
}

impl Scene for SplashScene {
    fn name(&self) -> &str {
        "Splash"
    }

    fn player(&self) -> Option<PipId> {
        self.player
    }

    fn register_tables(&self, tables: &mut Tables) {
        register_common(tables);
    }

    fn unregister_tables(&self, tables: &mut Tables) {
        unregister_common(tables);
    }

    fn populate(&mut self, registry: &AssetRegistry, domain: &mut Domain) {
        let mut rng = rand::rng();

        let player_pos = Vec2::ZERO;
        self.player = Some(domain.make(ActorBlueprint {
            xform: Transform { xyz: Vec3::new(player_pos.x, player_pos.y, 0.0), rot: glam::Quat::IDENTITY },
            brush: make_brush(registry, "player_happy", 5.0),
            name: Some("player".to_string()),
            motion: Some(Motion { vel: Vec3::ZERO }),
            team: Some(Team::Player),
            health: None,
            pilot: Some(PilotData { state: PilotState::Wander { goal: random_pos(&mut rng), timer: 1.0 }, speed: 3.0, cooldown: 0.0 }),
        }));

        let prop_count = rng.random_range(4usize..=6);
        for _ in 0..prop_count {
            let pos = random_pos(&mut rng);
            domain.make(ActorBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "cactus", 2.0),
                name: None,
                motion: None,
                team: Some(Team::Neutral),
                health: None,
                pilot: None,
            });
        }
    }

    fn setup(&mut self, scripts: &mut Scripts, solvers: &mut Solvers) {
        scripts.add(ScriptHost::new(
            EveryScript { enabled: true },
            Box::new(PilotScript),
        ));
        solvers.register(MovementSolver);
        solvers.register(BoundsSolver);
    }

    fn teardown(&self, _scripts: &mut Scripts, solvers: &mut Solvers) {
        let _ = solvers.remove::<MovementSolver>();
        let _ = solvers.remove::<BoundsSolver>();
    }

    fn is_complete(&mut self, dt: f32, _domain: &Domain) -> bool {
        self.timer += dt;
        self.timer > 8.0
    }
}

pub struct ArenaScene {
    timer: f32,
    player: Option<PipId>,
}

impl ArenaScene {
    pub fn new() -> Self {
        Self { timer: 0.0, player: None }
    }

    fn health_dead(&self, domain: &Domain) -> bool {
        let Some(pid) = self.player else { return false };
        let Some(ptr) = domain.ids.get(pid) else { return true };
        domain.tables.get::<HealthAddition>()
            .and_then(|h| h.data.get_row(ptr))
            .map_or(true, |h| h.health <= 0.0)
    }
}

impl Scene for ArenaScene {
    fn name(&self) -> &str {
        "Arena"
    }

    fn player(&self) -> Option<PipId> {
        self.player
    }

    fn register_tables(&self, tables: &mut Tables) {
        register_common(tables);
        if tables.get::<HealthAddition>().is_none() {
            tables.add(HealthAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<HealthData>()) });
        }
        if tables.get::<ProjectileAddition>().is_none() {
            tables.add(ProjectileAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<crate::arena::tables::ProjectileData>()) });
        }
        if tables.get::<SpawnerAddition>().is_none() {
            tables.add(SpawnerAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<SpawnerData>()) });
        }
        if tables.get::<HealthPickupAddition>().is_none() {
            tables.add(HealthPickupAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<HealthPickupData>()) });
        }
    }

    fn unregister_tables(&self, tables: &mut Tables) {
        let _ = tables.remove::<HealthAddition>();
        let _ = tables.remove::<ProjectileAddition>();
        let _ = tables.remove::<SpawnerAddition>();
        let _ = tables.remove::<HealthPickupAddition>();
        unregister_common(tables);
    }

    fn populate(&mut self, registry: &AssetRegistry, domain: &mut Domain) {
        let mut rng = rand::rng();

        let enemy_count = rng.random_range(4usize..=6);
        for _ in 0..enemy_count {
            let pos = random_pos(&mut rng);
            domain.make(ActorBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "red", 1.2),
                name: None,
                motion: Some(Motion { vel: Vec3::ZERO }),
                team: Some(Team::Enemy),
                health: Some(HealthData { health: 30.0, max: 30.0 }),
                pilot: Some(PilotData { state: PilotState::Wander { goal: random_pos(&mut rng), timer: 1.0 }, speed: 2.5, cooldown: 0.0 }),
            });
        }

        let player_pos = Vec2::ZERO;
        let player_state = PilotState::Wander { goal: random_pos(&mut rng), timer: 2.0 };
        self.player = Some(domain.make(ActorBlueprint {
            xform: Transform { xyz: Vec3::new(player_pos.x, player_pos.y, 0.0), rot: glam::Quat::IDENTITY },
            brush: make_brush(registry, "player_happy", 5.0),
            name: Some("player".to_string()),
            motion: Some(Motion { vel: Vec3::ZERO }),
            team: Some(Team::Player),
            health: Some(HealthData { health: 150.0, max: 150.0 }),
            pilot: Some(PilotData { state: player_state, speed: 4.0, cooldown: 0.0 }),
        }));

        let spawner_count = rng.random_range(1usize..=2);
        for _ in 0..spawner_count {
            let pos = random_pos(&mut rng);
            domain.make(SpawnerBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "blue", 1.2),
                name: None,
                team: Some(Team::Neutral),
                spawner: SpawnerData { interval: 2.0, timer: 1.0, max_count: 10, spawned: 0, enemy_brush: make_brush(registry, "red", 1.2) },
            });
        }

        let pickup_count = rng.random_range(1usize..=2);
        for _ in 0..pickup_count {
            let pos = random_pos(&mut rng);
            domain.make(PickupBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "townie_1", 3.0),
                name: None,
                team: Some(Team::Pickup),
                pickup: HealthPickupData { amount: 20.0 },
            });
        }

        let prop_count = rng.random_range(2usize..=4);
        for _ in 0..prop_count {
            let pos = random_pos(&mut rng);
            domain.make(ActorBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "cactus", 2.0),
                name: None,
                motion: None,
                team: Some(Team::Neutral),
                health: None,
                pilot: None,
            });
        }

        if let Some(p) = self.player {
            set_enemies_to_chase(domain, p);
        }
    }

    fn setup(&mut self, scripts: &mut Scripts, solvers: &mut Solvers) {
        scripts.add(ScriptHost::new(EveryScript { enabled: true }, Box::new(PilotScript)));
        solvers.register(MovementSolver);
        solvers.register(BoundsSolver);
        solvers.register(ProjectileSolver);
        solvers.register(SpawnerSolver { player: self.player });
        solvers.register(PickupSolver { player: self.player });
    }

    fn teardown(&self, _scripts: &mut Scripts, solvers: &mut Solvers) {
        let _ = solvers.remove::<MovementSolver>();
        let _ = solvers.remove::<BoundsSolver>();
        let _ = solvers.remove::<ProjectileSolver>();
        let _ = solvers.remove::<SpawnerSolver>();
        let _ = solvers.remove::<PickupSolver>();
    }

    fn is_complete(&mut self, dt: f32, domain: &Domain) -> bool {
        self.timer += dt;
        self.timer > 20.0 || self.health_dead(domain)
    }
}

pub struct SwarmScene {
    timer: f32,
    player: Option<PipId>,
}

impl SwarmScene {
    pub fn new() -> Self {
        Self { timer: 0.0, player: None }
    }

    fn health_dead(&self, domain: &Domain) -> bool {
        let Some(pid) = self.player else { return false };
        let Some(ptr) = domain.ids.get(pid) else { return true };
        domain.tables.get::<HealthAddition>()
            .and_then(|h| h.data.get_row(ptr))
            .map_or(true, |h| h.health <= 0.0)
    }
}

impl Scene for SwarmScene {
    fn name(&self) -> &str {
        "Swarm"
    }

    fn player(&self) -> Option<PipId> {
        self.player
    }

    fn register_tables(&self, tables: &mut Tables) {
        register_common(tables);
        if tables.get::<HealthAddition>().is_none() {
            tables.add(HealthAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<HealthData>()) });
        }
        if tables.get::<ProjectileAddition>().is_none() {
            tables.add(ProjectileAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<crate::arena::tables::ProjectileData>()) });
        }
        if tables.get::<SpawnerAddition>().is_none() {
            tables.add(SpawnerAddition { data: Class::new(rarity::common(()), GrowthStrategy::quart_kib::<SpawnerData>()) });
        }
    }

    fn unregister_tables(&self, tables: &mut Tables) {
        let _ = tables.remove::<HealthAddition>();
        let _ = tables.remove::<ProjectileAddition>();
        let _ = tables.remove::<SpawnerAddition>();
        unregister_common(tables);
    }

    fn populate(&mut self, registry: &AssetRegistry, domain: &mut Domain) {
        let mut rng = rand::rng();

        let player_pos = Vec2::ZERO;
        self.player = Some(domain.make(ActorBlueprint {
            xform: Transform { xyz: Vec3::new(player_pos.x, player_pos.y, 0.0), rot: glam::Quat::IDENTITY },
            brush: make_brush(registry, "player_happy", 5.0),
            name: Some("player".to_string()),
            motion: Some(Motion { vel: Vec3::ZERO }),
            team: Some(Team::Player),
            health: Some(HealthData { health: 250.0, max: 250.0 }),
            pilot: Some(PilotData { state: PilotState::Wander { goal: random_pos(&mut rng), timer: 2.0 }, speed: 4.0, cooldown: 0.0 }),
        }));

        let enemy_count = rng.random_range(20usize..=30);
        for _ in 0..enemy_count {
            let pos = random_pos(&mut rng);
            domain.make(ActorBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "bandit-1", 3.0),
                name: None,
                motion: Some(Motion { vel: Vec3::ZERO }),
                team: Some(Team::Enemy),
                health: Some(HealthData { health: 20.0, max: 20.0 }),
                pilot: Some(PilotData { state: PilotState::Wander { goal: random_pos(&mut rng), timer: 1.0 }, speed: 2.0, cooldown: 0.0 }),
            });
        }

        let spawner_count = rng.random_range(2usize..=3);
        for _ in 0..spawner_count {
            let pos = random_pos(&mut rng);
            domain.make(SpawnerBlueprint {
                xform: Transform { xyz: Vec3::new(pos.x, pos.y, 0.0), rot: glam::Quat::IDENTITY },
                brush: make_brush(registry, "yellow", 1.2),
                name: None,
                team: Some(Team::Neutral),
                spawner: SpawnerData { interval: 1.5, timer: 0.5, max_count: 40, spawned: 0, enemy_brush: make_brush(registry, "bandit-1", 3.0) },
            });
        }

        if let Some(p) = self.player {
            set_enemies_to_chase(domain, p);
        }
    }

    fn setup(&mut self, scripts: &mut Scripts, solvers: &mut Solvers) {
        scripts.add(ScriptHost::new(EveryScript { enabled: true }, Box::new(PilotScript)));
        solvers.register(MovementSolver);
        solvers.register(BoundsSolver);
        solvers.register(ProjectileSolver);
        solvers.register(SpawnerSolver { player: self.player });
    }

    fn teardown(&self, _scripts: &mut Scripts, solvers: &mut Solvers) {
        let _ = solvers.remove::<MovementSolver>();
        let _ = solvers.remove::<BoundsSolver>();
        let _ = solvers.remove::<ProjectileSolver>();
        let _ = solvers.remove::<SpawnerSolver>();
    }

    fn is_complete(&mut self, dt: f32, domain: &Domain) -> bool {
        self.timer += dt;
        self.timer > 30.0 || self.health_dead(domain)
    }
}
