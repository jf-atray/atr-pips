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
    pub struct TeamAddition as TeamView {
        pub team: Class<Team>,
    }
}

crate::partition! {
    pub struct PilotAddition as PilotView {
        pub data: Class<PilotData>,
    }
}

crate::partition! {
    pub struct HealthAddition as HealthView {
        pub data: Class<HealthData>,
    }
}

crate::partition! {
    pub struct ProjectileAddition as ProjectileView {
        pub data: Class<ProjectileData>,
    }
}

crate::partition! {
    pub struct SpawnerAddition as SpawnerView {
        pub data: Class<SpawnerData>,
    }
}

crate::partition! {
    pub struct HealthPickupAddition as HealthPickupView {
        pub data: Class<HealthPickupData>,
    }
}
