#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pulse {
    #[default]
    One,
    Two,
    Three,
    Go,
}

impl Pulse {
    pub fn advance(self) -> Self {
        match self {
            Pulse::One => Pulse::Two,
            Pulse::Two => Pulse::Three,
            Pulse::Three => Pulse::Go,
            Pulse::Go => Pulse::One,
        }
    }

    pub fn is_go(self) -> bool {
        self == Pulse::Go
    }
}

pub const PULSE_DURATION: f32 = 2.0 / 3.0;

pub const WALK_SPEED: f32 = 0.66;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PauseState {
    pub paused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverworldState {
    pub walk_speed: f32,
    pub dodge_speed_mul: f32,
    pub pulse: Pulse,
    pub pulse_timer: f32,
    pub duel_scurrying: bool,
    pub ground_y: f32,
    pub horizon_y: f32,
    pub suppress_chatter: bool,
    pub has_valid_targets: bool,
    pub possible_draw: bool,
    pub duel_running: bool,
    pub bad_guys_killed: u32,
}

impl Default for OverworldState {
    fn default() -> Self {
        Self {
            walk_speed: WALK_SPEED,
            dodge_speed_mul: 1.0,
            pulse: Pulse::One,
            pulse_timer: 0.0,
            duel_scurrying: false,
            ground_y: 3.5,
            horizon_y: 0.0,
            suppress_chatter: false,
            has_valid_targets: false,
            possible_draw: false,
            duel_running: false,
            bad_guys_killed: 0,
        }
    }
}
