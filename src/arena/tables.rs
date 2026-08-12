use glam::Vec2;

use crate::brushes::Brush;
use crate::tables::PipId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Team {
    Player,
    Enemy,
    Neutral,
    Pickup,
}

#[derive(Clone, Copy, Debug)]
pub struct HealthData {
    pub health: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum PilotState {
    Wander { goal: Vec2, timer: f32 },
    Chase { target: PipId },
}

#[derive(Clone, Copy, Debug)]
pub struct PilotData {
    pub state: PilotState,
    pub speed: f32,
    pub cooldown: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileData {
    pub lifetime: f32,
    pub damage: f32,
    pub owner: PipId,
}

#[derive(Clone, Debug)]
pub struct SpawnerData {
    pub interval: f32,
    pub timer: f32,
    pub max_count: u32,
    pub spawned: u32,
    pub enemy_brush: Brush,
}

#[derive(Clone, Copy, Debug)]
pub struct HealthPickupData {
    pub amount: f32,
}

crate::partition! {
    pub struct ActorAddition as ActorView {
        pub team: Class<Team>,
        pub pilot: Class<PilotData>,
    }
}

impl ActorView {
    pub fn team(&mut self, team: Team) -> &mut Self {
        self.team = Some(team);
        self
    }

    pub fn pilot(&mut self, team: Team, pilot: PilotData) -> &mut Self {
        self.team = Some(team);
        self.pilot = Some(pilot);
        self
    }
}

crate::partition! {
    pub struct ArenaAddition as ArenaView {
        pub health: Class<HealthData>,
        pub projectile: Class<ProjectileData>,
        pub spawner: Class<SpawnerData>,
        pub pickup: Class<HealthPickupData>,
    }
}

impl ArenaView {
    pub fn health(&mut self, health: HealthData) -> &mut Self {
        self.health = Some(health);
        self
    }

    pub fn projectile(&mut self, projectile: ProjectileData) -> &mut Self {
        self.projectile = Some(projectile);
        self
    }

    pub fn spawner(&mut self, spawner: SpawnerData) -> &mut Self {
        self.spawner = Some(spawner);
        self
    }

    pub fn pickup(&mut self, pickup: HealthPickupData) -> &mut Self {
        self.pickup = Some(pickup);
        self
    }
}
