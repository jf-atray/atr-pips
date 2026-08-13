use glam::{Vec2, Vec3};
use rand::Rng;

use crate::arena::scripts::PilotScript;
use crate::arena::solvers::{
    BoundsSolver, MovementSolver, PickupSolver, ProjectileSolver, SpawnerSolver,
};
use crate::arena::tables::{
    ActorAddition, ActorView, ArenaAddition, ArenaView, HealthData, HealthPickupData, PilotData,
    PilotState, SpawnerData, Team,
};
use crate::assets::AssetRegistry;
use crate::brushes::Brush;
use crate::gamescope::scene::Scene;
use crate::query::impls::query_mut_mut;
use crate::scripting::{EveryScript, ScriptHost, Scripts, Solvers};
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::PipId;
use crate::tables::domain::Domain;
use crate::tables::scope::Scope;
use crate::tables::tables::Tables;

const SPAWN_BOUNDS: f32 = 12.0;

fn make_brush(registry: &AssetRegistry, name: &str, scale: f32) -> Brush {
    let e = registry.get(name);
    let mut brush = Brush::from_sprite(e);
    brush.scale *= scale;
    brush
}

fn random_pos(rng: &mut impl Rng) -> Vec2 {
    Vec2::new(
        rng.random::<f32>() * SPAWN_BOUNDS * 2.0 - SPAWN_BOUNDS,
        rng.random::<f32>() * SPAWN_BOUNDS * 2.0 - SPAWN_BOUNDS,
    )
}

fn register_common(tables: &mut Tables) {
    tables.get_or_insert::<ActorAddition>(ActorAddition::new());
}

fn unregister_common(tables: &mut Tables) {
    let _ = tables.remove::<ActorAddition>();
}

fn player_dead(domain: &Domain, player: PipId) -> bool {
    let Some(ptr) = domain.ids.get(player) else {
        return true;
    };
    domain
        .tables
        .get::<ArenaAddition>()
        .and_then(|a| a.health.get_row(ptr))
        .is_none_or(|h| h.health <= 0.0)
}

fn set_enemies_to_chase(domain: &mut Domain, player: PipId) {
    let view = &mut domain.tables.view();
    let actor = view.additions.get_mut::<ActorAddition>().unwrap();

    for (pilot_col, team_col) in query_mut_mut(&mut actor.pilot, &(), &mut actor.team, &()) {
        for i in 0..pilot_col.len() {
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
        Self {
            timer: 0.0,
            player: None,
        }
    }
}

impl Scene for SplashScene {
    fn name(&self) -> &'static str {
        "Splash"
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
        let prop_count = rng.random_range(4usize..=6);
        let random_goal = random_pos(&mut rng);

        self.player = Some(domain.make(move |scope: &mut Scope| {
            scope.core.with(
                Transform {
                    xyz: Vec3::new(player_pos.x, player_pos.y, 0.0),
                    rot: glam::Quat::IDENTITY,
                },
                make_brush(registry, "player_happy", 5.0),
                "player".to_string(),
                Motion { vel: Vec3::ZERO },
            );
            scope.view::<ActorView>().map(|view| view.pilot(Team::Player, PilotData {
                state: PilotState::Wander {
                    goal: random_goal,
                    timer: 1.0,
                },
                speed: 3.0,
                cooldown: 0.0,
            }));
        }));

        for _ in 0..prop_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "cactus", 2.0);
            domain.make(move |scope: &mut Scope| {
                scope.core.xforms = Some(xform);
                scope.core.brushes = Some(brush);
                scope.view::<ActorView>().map(|view| view.team(Team::Neutral));
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
        Self {
            timer: 0.0,
            player: None,
        }
    }

    fn health_dead(&self, domain: &Domain) -> bool {
        self.player.is_some_and(|p| player_dead(domain, p))
    }
}

impl Scene for ArenaScene {
    fn name(&self) -> &'static str {
        "Arena"
    }


    fn register_tables(&self, tables: &mut Tables) {
        register_common(tables);
        tables.get_or_insert::<ArenaAddition>(ArenaAddition::new());
    }

    fn unregister_tables(&self, tables: &mut Tables) {
        let _ = tables.remove::<ArenaAddition>();
        unregister_common(tables);
    }

    fn populate(&mut self, registry: &AssetRegistry, domain: &mut Domain) {
        let mut rng = rand::rng();

        let enemy_count = rng.random_range(4usize..=6);
        for _ in 0..enemy_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "red", 1.2);
            let health = HealthData {
                health: 30.0,
                max: 30.0,
            };
            let pilot = PilotData {
                state: PilotState::Wander {
                    goal: random_pos(&mut rng),
                    timer: 1.0,
                },
                speed: 2.5,
                cooldown: 0.0,
            };
            domain.make(move |scope: &mut Scope| {
                scope.core.with(xform, brush, String::new(), Motion::default());
                scope.view::<ActorView>().map(|view| view.pilot(Team::Enemy, pilot));
                scope.view::<ArenaView>().map(|view| view.health(health));
            });
        }

        let player_pos = Vec2::ZERO;
        let player_state = PilotState::Wander {
            goal: random_pos(&mut rng),
            timer: 2.0,
        };
        let player_xform = Transform {
            xyz: Vec3::new(player_pos.x, player_pos.y, 0.0),
            rot: glam::Quat::IDENTITY,
        };
        let player_brush = make_brush(registry, "player_happy", 5.0);
        let player_health = HealthData {
            health: 150.0,
            max: 150.0,
        };
        let player_pilot = PilotData {
            state: player_state,
            speed: 4.0,
            cooldown: 0.0,
        };
        self.player = Some(domain.make(move |scope: &mut Scope| {
            scope.core.with(
                player_xform,
                player_brush,
                "player".to_string(),
                Motion { vel: Vec3::ZERO },
            );
            scope.view::<ActorView>().map(|view| view.pilot(Team::Player, player_pilot));
            scope.view::<ArenaView>().map(|view| view.health(player_health));
        }));

        let spawner_count = rng.random_range(1usize..=2);
        for _ in 0..spawner_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "blue", 1.2);
            let enemy_brush = make_brush(registry, "red", 1.2);
            let spawner = SpawnerData {
                interval: 2.0,
                timer: 1.0,
                max_count: 10,
                spawned: 0,
                enemy_brush,
            };
            domain.make(move |scope: &mut Scope| {
                scope.core.xforms = Some(xform);
                scope.core.brushes = Some(brush);
                scope.view::<ActorView>().map(|view| view.team(Team::Neutral));
                scope.view::<ArenaView>().map(|view| view.spawner(spawner));
            });
        }

        let pickup_count = rng.random_range(1usize..=2);
        for _ in 0..pickup_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "townie_1", 3.0);
            let pickup = HealthPickupData { amount: 20.0 };
            domain.make(move |scope: &mut Scope| {
                scope.core.xforms = Some(xform);
                scope.core.brushes = Some(brush);
                scope.view::<ActorView>().map(|view| view.team(Team::Pickup));
                scope.view::<ArenaView>().map(|view| view.pickup(pickup));
            });
        }

        let prop_count = rng.random_range(2usize..=4);
        for _ in 0..prop_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "cactus", 2.0);
            domain.make(move |scope: &mut Scope| {
                scope.core.xforms = Some(xform);
                scope.core.brushes = Some(brush);
                scope.view::<ActorView>().map(|view| view.team(Team::Neutral));
            });
        }

        if let Some(p) = self.player {
            set_enemies_to_chase(domain, p);
        }
    }

    fn setup(&mut self, scripts: &mut Scripts, solvers: &mut Solvers) {
        scripts.add(ScriptHost::new(
            EveryScript { enabled: true },
            Box::new(PilotScript),
        ));
        solvers.register(MovementSolver);
        solvers.register(BoundsSolver);
        solvers.register(ProjectileSolver);
        solvers.register(SpawnerSolver {
            player: self.player,
        });
        solvers.register(PickupSolver {
            player: self.player,
        });
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
        Self {
            timer: 0.0,
            player: None,
        }
    }
}

impl Scene for SwarmScene {
    fn name(&self) -> &'static str {
        "Swarm"
    }

    fn register_tables(&self, tables: &mut Tables) {
        register_common(tables);
        tables.get_or_insert::<ArenaAddition>(ArenaAddition::new());
    }

    fn unregister_tables(&self, tables: &mut Tables) {
        let _ = tables.remove::<ArenaAddition>();
        unregister_common(tables);
    }

    fn populate(&mut self, registry: &AssetRegistry, domain: &mut Domain) {
        let mut rng = rand::rng();

        let player_pos = Vec2::ZERO;
        let player_xform = Transform {
            xyz: Vec3::new(player_pos.x, player_pos.y, 0.0),
            rot: glam::Quat::IDENTITY,
        };
        let player_brush = make_brush(registry, "player_happy", 5.0);
        let player_health = HealthData {
            health: 250.0,
            max: 250.0,
        };
        let player_pilot = PilotData {
            state: PilotState::Wander {
                goal: random_pos(&mut rng),
                timer: 2.0,
            },
            speed: 4.0,
            cooldown: 0.0,
        };
        self.player = Some(domain.make(move |scope: &mut Scope| {
            scope.core.with(
                player_xform,
                player_brush,
                "player".to_string(),
                Motion { vel: Vec3::ZERO },
            );
            scope.view::<ActorView>().map(|view| view.pilot(Team::Player, player_pilot));
            scope.view::<ArenaView>().map(|view| view.health(player_health));
        }));

        let enemy_count = rng.random_range(20usize..=30);
        for _ in 0..enemy_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "bandit-1", 3.0);
            let health = HealthData {
                health: 20.0,
                max: 20.0,
            };
            let pilot = PilotData {
                state: PilotState::Wander {
                    goal: random_pos(&mut rng),
                    timer: 1.0,
                },
                speed: 2.0,
                cooldown: 0.0,
            };
            domain.make(move |scope: &mut Scope| {
                scope.core.xforms = Some(xform);
                scope.core.brushes = Some(brush);
                scope.core.motions = Some(Motion { vel: Vec3::ZERO });
                scope.view::<ActorView>().map(|view| view.pilot(Team::Enemy, pilot));
                scope.view::<ArenaView>().map(|view| view.health(health));
            });
        }

        let spawner_count = rng.random_range(2usize..=3);
        for _ in 0..spawner_count {
            let pos = random_pos(&mut rng);
            let xform = Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: glam::Quat::IDENTITY,
            };
            let brush = make_brush(registry, "yellow", 1.2);
            let enemy_brush = make_brush(registry, "bandit-1", 3.0);
            let spawner = SpawnerData {
                interval: 1.5,
                timer: 0.5,
                max_count: 40,
                spawned: 0,
                enemy_brush,
            };
            domain.make(move |scope: &mut Scope| {
                scope.core.xforms = Some(xform);
                scope.core.brushes = Some(brush);
                scope.view::<ActorView>().map(|view| view.team(Team::Neutral));
                scope.view::<ArenaView>().map(|view| view.spawner(spawner));
            });
        }

        if let Some(p) = self.player {
            set_enemies_to_chase(domain, p);
        }
    }

    fn setup(&mut self, scripts: &mut Scripts, solvers: &mut Solvers) {
        scripts.add(ScriptHost::new(
            EveryScript { enabled: true },
            Box::new(PilotScript),
        ));
        solvers.register(MovementSolver);
        solvers.register(BoundsSolver);
        solvers.register(ProjectileSolver);
        solvers.register(SpawnerSolver {
            player: self.player,
        });
    }

    fn teardown(&self, _scripts: &mut Scripts, solvers: &mut Solvers) {
        let _ = solvers.remove::<MovementSolver>();
        let _ = solvers.remove::<BoundsSolver>();
        let _ = solvers.remove::<ProjectileSolver>();
        let _ = solvers.remove::<SpawnerSolver>();
    }

    fn is_complete(&mut self, dt: f32, domain: &Domain) -> bool {
        self.timer += dt;
        self.timer > 30.0 || self.player.is_some_and(|p| player_dead(domain, p))
    }
}
