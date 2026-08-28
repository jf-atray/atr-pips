use crate::addition;
use crate::ecs::class::Class;
use crate::ecs::class_strategy::GrowthStrategy;
use super::dm::DungeonMaster;
use super::state::DuelState;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DuelEnemy {
    pub wave: u32,
    pub active: bool,
    pub fire_countdown: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DuelReticle {
    pub lateral: f32,
    pub d: f32,
    pub speed: f32,
    pub sway_phase: f32,
    pub snapped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DuelCursor {
    pub lateral: f32,
}

//todo what an atrocious tables layout I had
addition! {
    #[derive(Debug)]
    pub struct duel_world : DuelWorld {
        tables: {
            duel_enemies: Class<DuelEnemy> = Class::new(GrowthStrategy::quart_kib::<DuelEnemy>()),
            duel_reticles: Class<DuelReticle> = Class::new(GrowthStrategy::quart_kib::<DuelReticle>()),
            duel_cursors: Class<DuelCursor> = Class::new(GrowthStrategy::quart_kib::<DuelCursor>()),
        },
        solvers: {
            dungeon_master: DungeonMaster = DungeonMaster::new(None, None, None),
        },
        scripts: {},
        signals: {
            duel_state: DuelState = DuelState::Inactive,
        },
    }
}