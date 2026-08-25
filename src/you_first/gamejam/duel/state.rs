//! Runtime state machine and beat timer for the duel minigame.

use crate::anims::{AnimId, AnimLibId};
use crate::ecs::PipId;
use crate::you_first::gamejam::duel::formation::Formation;
use crate::you_first::gamejam::roller::state::Pulse;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadGuyKind {
    Normal,
    Frozen { kills: u32, howdys: u32 },
    Offscreen {
        kills: u32,
        next: Box<BadGuyKind>,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ReticlePattern {
    #[default]
    Tracking,
    Linear {
        start_offset: Vec2,
        goal_offset: Vec2,
        duration: f32,
        panic_kills: Option<u32>,
    },
}

#[derive(Debug, Clone)]
pub struct BadGuySpec {
    pub kind: BadGuyKind,
    pub start_delay: u32,
    pub aim_timeout: u32,
    pub recover_beats: u32,
    pub reticle_pattern: ReticlePattern,
}

impl BadGuySpec {
    pub fn normal() -> Self {
        Self {
            kind: BadGuyKind::Normal,
            start_delay: 0,
            aim_timeout: 5,
            recover_beats: 1,
            reticle_pattern: ReticlePattern::Tracking,
        }
    }
    pub fn frozen(kills: u32) -> Self {
        Self {
            kind: BadGuyKind::Frozen { kills, howdys: 0 },
            start_delay: 0,
            aim_timeout: 7,
            recover_beats: 1,
            reticle_pattern: ReticlePattern::Tracking,
        }
    }
    pub fn offscreen(kills: u32, next: BadGuyKind) -> Self {
        Self {
            kind: BadGuyKind::Offscreen {
                kills,
                next: Box::new(next),
            },
            start_delay: 0,
            aim_timeout: 7,
            recover_beats: 1,
            reticle_pattern: ReticlePattern::Tracking,
        }
    }
    pub fn with_start_delay(mut self, delay: u32) -> Self {
        self.start_delay = delay;
        self
    }
    pub fn with_aim_timeout(mut self, timeout: u32) -> Self {
        self.aim_timeout = timeout;
        self
    }
    pub fn with_reticle_pattern(mut self, pattern: ReticlePattern) -> Self {
        self.reticle_pattern = pattern;
        self
    }
    pub fn with_howdy_threshold(mut self, howdys: u32) -> Self {
        if let BadGuyKind::Frozen { kills, .. } = &mut self.kind {
            self.kind = BadGuyKind::Frozen { kills: *kills, howdys };
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownspersonKind {
    Normal,
    Frozen { kills: u32, howdys: u32 },
    Offscreen {
        kills: u32,
        howdys: u32,
        next: Box<TownspersonKind>,
    },
}

impl TownspersonKind {
    pub fn is_gated(&self) -> bool {
        matches!(
            self,
            TownspersonKind::Frozen { .. } | TownspersonKind::Offscreen { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct TownspersonSpec {
    pub kind: TownspersonKind,
    pub side: Side,
    pub timeout: u32,
}

impl TownspersonSpec {
    pub fn normal(side: Side) -> Self {
        Self {
            kind: TownspersonKind::Normal,
            side,
            timeout: u32::MAX,
        }
    }
    pub fn frozen(side: Side, kills: u32) -> Self {
        Self {
            kind: TownspersonKind::Frozen { kills, howdys: 0 },
            side,
            timeout: u32::MAX,
        }
    }

    pub fn offscreen(side: Side, kills: u32, howdys: u32, next: TownspersonKind) -> Self {
        Self {
            kind: TownspersonKind::Offscreen {
                kills,
                howdys,
                next: Box::new(next),
            },
            side,
            timeout: u32::MAX,
        }
    }
    pub fn with_howdy_threshold(mut self, howdys: u32) -> Self {
        match &mut self.kind {
            TownspersonKind::Frozen { kills, .. } => {
                self.kind = TownspersonKind::Frozen { kills: *kills, howdys };
            }
            TownspersonKind::Offscreen { kills, next, .. } => {
                self.kind = TownspersonKind::Offscreen {
                    kills: *kills,
                    howdys,
                    next: next.clone(),
                };
            }
            _ => {}
        }
        self
    }
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ChallengeConfig {
    pub bad_guys: Vec<BadGuySpec>,
    pub bad_guy_formation: Formation,
    pub townspeople: Vec<TownspersonSpec>,
    pub townsperson_formation: Formation,
}

#[derive(Debug)]
pub struct PendingDuel {
    pub bad_guys: Vec<(PipId, BadGuySpec)>,
    pub townspeople: Vec<(PipId, TownspersonSpec)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerupState {
    #[default]
    None,
    Howdy,
    SixShot,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SayIntent {
    #[default]
    None,
    Count(Pulse),
    Draw,
    Howdy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HowdyPerfectRun(pub bool);

impl HowdyPerfectRun {
    pub fn new() -> Self {
        Self(true)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuelPhase {
    ZoomOut,
    Active,
    Returning,
    ZoomIn,
    ScurryLoss,
    Done,
    #[default]
    Idle,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatTimer {
    pub beat_duration: f32,
    pub elapsed: f32,
    pub beat_index: u32,
    pub beat_this_tick: bool,
}

impl BeatTimer {
    pub fn new(beat_duration: f32) -> Self {
        Self {
            beat_duration,
            elapsed: 0.0,
            beat_index: 0,
            beat_this_tick: false,
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        if self.elapsed >= self.beat_duration {
            self.elapsed -= self.beat_duration;
            self.beat_index += 1;
            self.beat_this_tick = true;
            true
        } else {
            self.beat_this_tick = false;
            false
        }
    }

    pub fn is_fire_beat(&self) -> bool {
        self.beat_index > 0 && self.beat_index.is_multiple_of(3)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuelOutcome {
    #[default]
    None,
    Win,
    Loss,
}

#[derive(Debug, Clone)]
pub struct HatAnimLib {
    pub lib: AnimLibId,
    pub rise_anim: AnimId,
}

#[derive(Debug, Clone)]
pub struct SpinAnimLib {
    pub lib: AnimLibId,
    pub spin_anim: AnimId,
}
#[derive(Debug, Clone, Default)]
pub struct ReticleAnimLib {
    pub lib: AnimLibId,
    pub slow_anim: AnimId,
    pub fast_anim: AnimId,
}

#[derive(Debug, Clone, Default)]
pub struct LivingAnimLib {
    pub lib: AnimLibId,
    pub root_anim: AnimId,
}

#[derive(Debug, Clone, Default)]
pub struct TumbleweedAnimLib {
    pub lib: AnimLibId,
    pub root_anim: AnimId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DuelState {
    #[default]
    Inactive,
    Requested,
    Active,
    Done(DuelOutcome),
}

#[derive(Debug, Clone, Copy)]
pub struct RetryState {
    pub challenge_index: usize,
    pub player_d: f32,
}

#[derive(Debug)]
pub struct Duel {
    pub state: DuelState,
    pub pending: Option<PendingDuel>,
    pub phase: DuelPhase,
    pub timer: f32,
}

impl Duel {
    pub fn new() -> Self {
        Self {
            state: DuelState::Inactive,
            pending: None,
            phase: DuelPhase::Idle,
            timer: 0.0,
        }
    }

    pub fn request(&mut self, pending: PendingDuel) {
        if matches!(self.state, DuelState::Inactive) {
            self.state = DuelState::Requested;
            self.pending = Some(pending);
            self.phase = DuelPhase::ZoomOut;
            self.timer = 0.0;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        match self.phase {
            DuelPhase::ZoomOut => {
                if matches!(self.state, DuelState::Requested) {
                    self.pending = None;
                    self.state = DuelState::Active;
                    self.phase = DuelPhase::Active;
                    self.timer = 0.0;
                }
            }
            DuelPhase::Active => {
                self.timer += dt;
                if self.timer >= 2.0 {
                    self.state = DuelState::Done(DuelOutcome::Win);
                    self.phase = DuelPhase::Returning;
                    self.timer = 0.0;
                }
            }
            DuelPhase::Returning => {
                self.timer += dt;
                if self.timer >= 0.5 {
                    self.phase = DuelPhase::ZoomIn;
                    self.timer = 0.0;
                }
            }
            DuelPhase::ZoomIn => {
                self.timer += dt;
                if self.timer >= 0.5 {
                    self.state = DuelState::Inactive;
                    self.phase = DuelPhase::Idle;
                    self.timer = 0.0;
                }
            }
            DuelPhase::ScurryLoss => {}
            DuelPhase::Done => {}
            DuelPhase::Idle => {
                if matches!(self.state, DuelState::Requested) {
                    self.phase = DuelPhase::ZoomOut;
                }
            }
        }
    }
}