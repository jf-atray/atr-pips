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

crate::partition! {
    pub struct DuelAddition as DuelView {
        pub duel_enemies: Class<DuelEnemy>,
        pub duel_reticles: Class<DuelReticle>,
        pub duel_cursors: Class<DuelCursor>,
    }
}
