use std::collections::HashMap;

use glam::{Vec2, Vec3, Vec4};

use crate::addition::{Addition, Ids, Pips, ScriptsMap, SignalsMap, Solver};
use crate::anims::{AnimId, AnimLibId, AnimWorld};
use crate::assets::SpriteEntry;
use crate::ecs::PipId;
use crate::ecs::class::Class;
use crate::gather::impls::{gather_mut, gather_ref};
use crate::input::Input;
use crate::you_first::gamejam::duel::bundle::duel_reticle_bundle;
use crate::you_first::gamejam::duel::components::{DuelReticle, DuelWorld};
use crate::you_first::gamejam::duel::formation::Formation;
use crate::you_first::gamejam::duel::state::{
    BadGuyKind, BadGuySpec, ChallengeConfig, Duel, DuelPhase, HowdyPerfectRun, LivingAnimLib,
    PendingDuel, ReticleAnimLib, ReticlePattern, RetryState, Side, TownspersonKind, TownspersonSpec,
};
use crate::you_first::gamejam::roller::bundles::living_roller_bundle;
use crate::you_first::gamejam::roller::components::{RollerDepth, RollerWorld};
use crate::you_first::gamejam::roller::projection::{
    DUEL_SPAWN_DISTANCE, DUEL_TRIGGER_DISTANCE,
};
use crate::you_first::gamejam::stats::GameStats;

const POST_DUEL_DELAY: f32 = 2.6;

const WAVE_DURATION: f32 = (2.0 * 2.0 / 3.0) * 1.5;
const WAVE_TRAVEL: f32 = 5.5;

fn linear_wave(
    angle: f32,
    count: usize,
    spread_start: f32,
    spread_end: f32,
    panic_kills: Option<u32>,
) -> Vec<ReticlePattern> {
    const EXTRA_SPACING: f32 = 0.4;
    let forward = Vec2::new(angle.cos(), angle.sin());
    let perp = Vec2::new(-angle.sin(), angle.cos());
    let extra = if count > 1 {
        EXTRA_SPACING * count as f32 * 0.5
    } else {
        0.0
    };
    let lo = spread_start - extra;
    let hi = spread_end + extra;
    (0..count)
        .map(|i| {
            let t = if count > 1 {
                i as f32 / (count - 1) as f32
            } else {
                0.5
            };
            let offset = lo + (hi - lo) * t;
            ReticlePattern::Linear {
                start_offset: forward * WAVE_TRAVEL + perp * offset,
                goal_offset: -forward * WAVE_TRAVEL + perp * offset,
                duration: WAVE_DURATION,
                panic_kills,
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DmPhase {
    WaitingToSpawn,
    ChallengeActive,
    DuelInProgress,
    PostDuelDelay,
    Finished,
}

#[derive(Debug)]
pub struct DungeonMaster {
    pub player: Option<PipId>,
    pub living_anim: Option<LivingAnimLib>,
    pub reticle_anim: Option<ReticleAnimLib>,
    pub duel: Duel,
    pub game_stats: GameStats,
    pub howdy_perfect_run: HowdyPerfectRun,
    retry: Option<RetryState>,
    phase: DmPhase,
    challenge_index: usize,
    challenges: Vec<ChallengeConfig>,
    spawned: Vec<(PipId, BadGuySpec)>,
    spawned_townspeople: Vec<(PipId, TownspersonSpec)>,
    delay_timer: f32,
}

impl DungeonMaster {
    pub fn new(
        player: Option<PipId>,
        living_anim: Option<LivingAnimLib>,
        reticle_anim: Option<ReticleAnimLib>,
        retry: Option<RetryState>,
    ) -> Self {
        Self {
            player,
            living_anim,
            reticle_anim,
            duel: Duel::new(),
            game_stats: GameStats::default(),
            howdy_perfect_run: HowdyPerfectRun::new(),
            retry,
            phase: DmPhase::WaitingToSpawn,
            challenge_index: 0,
            challenges: Self::build_challenges(),
            spawned: Vec::new(),
            spawned_townspeople: Vec::new(),
            delay_timer: 0.0,
        }
    }

    pub fn challenge_index(&self) -> usize {
        self.challenge_index
    }

    pub fn total_challenges(&self) -> usize {
        self.challenges.len()
    }

    pub fn is_finished(&self) -> bool {
        self.phase == DmPhase::Finished
    }

    fn build_challenges() -> Vec<ChallengeConfig> {
        use std::f32::consts::TAU;

        let wall_r2 = linear_wave(0.0, 8, -3.5, 0.0, Some(12));
        let wall_r3 = linear_wave(0.0, 6, -1.5, 0.0, None);
        let wave_r4_v = linear_wave(TAU * 0.25, 9, -3.5, 0.0, None);
        let wave_r4_h = linear_wave(TAU * 0.5, 6, -3.5, 0.0, Some(10));

        let boss_waves: Vec<Vec<ReticlePattern>> = (0..7)
            .map(|seg| {
                let angle = if seg % 2 == 0 { 0.0 } else { TAU * 0.25 };
                let panic = if seg < 6 {
                    Some((seg + 1) as u32 * 6 - 1)
                } else {
                    None
                };
                linear_wave(angle, 7, -3.0, 0.0, panic)
            })
            .collect();

        vec![
            ChallengeConfig {
                bad_guys: vec![
                    BadGuySpec::normal(),
                    BadGuySpec::offscreen(1, BadGuyKind::Normal),
                    BadGuySpec::offscreen(1, BadGuyKind::Normal),
                ],
                bad_guy_formation: Formation::SemiCircle { radius: 2.2 },
                townspeople: vec![],
                townsperson_formation: Formation::SemiCircle { radius: 3.2 },
            },
            ChallengeConfig {
                bad_guys: vec![],
                bad_guy_formation: Formation::SemiCircle { radius: 3.2 },
                townspeople: vec![TownspersonSpec::normal(Side::Right)],
                townsperson_formation: Formation::SemiCircle { radius: 3.2 },
            },
            ChallengeConfig {
                bad_guys: vec![
                    BadGuySpec::normal(),
                    BadGuySpec::normal().with_start_delay(1),
                    BadGuySpec::normal().with_start_delay(2),
                    BadGuySpec::frozen(1),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 }),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 3, howdys: 0 })
                        .with_start_delay(1),
                    BadGuySpec::offscreen(2, BadGuyKind::Frozen { kills: 4, howdys: 0 }),
                    BadGuySpec::offscreen(3, BadGuyKind::Frozen { kills: 4, howdys: 0 })
                        .with_start_delay(1),
                    BadGuySpec::offscreen(4, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[0].clone()),
                    BadGuySpec::offscreen(4, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[1].clone()),
                    BadGuySpec::offscreen(5, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[2].clone()),
                    BadGuySpec::offscreen(5, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[3].clone()),
                    BadGuySpec::offscreen(6, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[4].clone()),
                    BadGuySpec::offscreen(6, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[5].clone()),
                    BadGuySpec::offscreen(6, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[6].clone()),
                    BadGuySpec::offscreen(6, BadGuyKind::Frozen { kills: 7, howdys: 0 })
                        .with_reticle_pattern(wall_r2[7].clone()),
                ],
                bad_guy_formation: Formation::SemiCircle { radius: 3.2 },
                townspeople: vec![TownspersonSpec::offscreen(
                    Side::Right,
                    2,
                    0,
                    TownspersonKind::Frozen { kills: 4, howdys: 0 },
                )
                .with_timeout(8)],
                townsperson_formation: Formation::SemiCircle { radius: 3.2 },
            },
            ChallengeConfig {
                bad_guys: vec![
                    BadGuySpec::normal(),
                    BadGuySpec::normal().with_start_delay(1),
                    BadGuySpec::normal().with_start_delay(2),
                    BadGuySpec::normal(),
                    BadGuySpec::frozen(2).with_reticle_pattern(wall_r3[0].clone()),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 })
                        .with_reticle_pattern(wall_r3[1].clone()),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 })
                        .with_reticle_pattern(wall_r3[2].clone()),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 })
                        .with_reticle_pattern(wall_r3[3].clone()),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 })
                        .with_reticle_pattern(wall_r3[4].clone()),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 })
                        .with_reticle_pattern(wall_r3[5].clone()),
                ],
                bad_guy_formation: Formation::SemiCircle { radius: 3.2 },
                townspeople: vec![TownspersonSpec::offscreen(
                    Side::Right,
                    2,
                    0,
                    TownspersonKind::Normal,
                )
                .with_timeout(5)],
                townsperson_formation: Formation::SemiCircle { radius: 3.2 },
            },
            ChallengeConfig {
                bad_guys: vec![
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[0].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[1].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[2].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[3].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[4].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[5].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[6].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[7].clone()),
                    BadGuySpec::normal().with_reticle_pattern(wave_r4_v[8].clone()),
                    BadGuySpec::frozen(1),
                    BadGuySpec::offscreen(1, BadGuyKind::Frozen { kills: 2, howdys: 0 }),
                    BadGuySpec::offscreen(2, BadGuyKind::Frozen { kills: 3, howdys: 0 }),
                    BadGuySpec::offscreen(2, BadGuyKind::Frozen { kills: 3, howdys: 0 }),
                    BadGuySpec::offscreen(4, BadGuyKind::Normal)
                        .with_reticle_pattern(wave_r4_h[0].clone()),
                    BadGuySpec::offscreen(4, BadGuyKind::Normal)
                        .with_reticle_pattern(wave_r4_h[1].clone()),
                    BadGuySpec::offscreen(4, BadGuyKind::Normal)
                        .with_reticle_pattern(wave_r4_h[2].clone()),
                    BadGuySpec::offscreen(4, BadGuyKind::Normal)
                        .with_reticle_pattern(wave_r4_h[3].clone()),
                    BadGuySpec::offscreen(4, BadGuyKind::Normal)
                        .with_reticle_pattern(wave_r4_h[4].clone()),
                    BadGuySpec::offscreen(4, BadGuyKind::Normal)
                        .with_reticle_pattern(wave_r4_h[5].clone()),
                ],
                bad_guy_formation: Formation::SemiCircle { radius: 2.5 },
                townspeople: vec![
                    TownspersonSpec::offscreen(
                        Side::Right,
                        1,
                        0,
                        TownspersonKind::Frozen { kills: 2, howdys: 0 },
                    )
                    .with_timeout(8),
                    TownspersonSpec::offscreen(Side::Left, 7, 0, TownspersonKind::Normal)
                        .with_timeout(12),
                ],
                townsperson_formation: Formation::SemiCircle { radius: 4.0 },
            },
            ChallengeConfig {
                bad_guys: {
                    let mut guys = Vec::new();
                    for (seg, wave) in boss_waves.iter().enumerate().take(7) {
                        let enter = (seg * 6) as u32;
                        let unfreeze = enter + 1;
                        for pattern in wave {
                            let mut spec = if seg == 0 {
                                BadGuySpec::normal()
                            } else {
                                BadGuySpec::offscreen(
                                    enter - 1,
                                    BadGuyKind::Frozen { kills: unfreeze, howdys: 0 },
                                )
                            };
                            spec = spec.with_aim_timeout(3).with_reticle_pattern(pattern.clone());
                            guys.push(spec);
                        }
                    }
                    guys
                },
                bad_guy_formation: Formation::SemiCircle { radius: 4.5 },
                townspeople: vec![],
                townsperson_formation: Formation::SemiCircle { radius: 3.2 },
            },
            ChallengeConfig {
                bad_guys: vec![],
                bad_guy_formation: Formation::SemiCircle { radius: 3.2 },
                townspeople: vec![
                    TownspersonSpec::normal(Side::Left),
                    TownspersonSpec::normal(Side::Left),
                    TownspersonSpec::normal(Side::Right),
                    TownspersonSpec::offscreen(
                        Side::Right,
                        0,
                        1,
                        TownspersonKind::Frozen { kills: 0, howdys: 2 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Left,
                        0,
                        2,
                        TownspersonKind::Frozen { kills: 0, howdys: 3 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Right,
                        0,
                        2,
                        TownspersonKind::Frozen { kills: 0, howdys: 3 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Left,
                        0,
                        2,
                        TownspersonKind::Frozen { kills: 0, howdys: 4 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Left,
                        0,
                        3,
                        TownspersonKind::Frozen { kills: 0, howdys: 4 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Right,
                        0,
                        3,
                        TownspersonKind::Frozen { kills: 0, howdys: 4 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Right,
                        0,
                        3,
                        TownspersonKind::Frozen { kills: 0, howdys: 5 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Left,
                        0,
                        3,
                        TownspersonKind::Frozen { kills: 0, howdys: 5 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Right,
                        0,
                        4,
                        TownspersonKind::Frozen { kills: 0, howdys: 5 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Left,
                        0,
                        5,
                        TownspersonKind::Frozen { kills: 0, howdys: 6 },
                    ),
                    TownspersonSpec::offscreen(
                        Side::Right,
                        0,
                        13,
                        TownspersonKind::Frozen { kills: 0, howdys: 13 },
                    ),
                ],
                townsperson_formation: Formation::SemiCircle { radius: 3.2 },
            },
        ]
    }

    fn spawn_challenge(
        &mut self,
        pips: &mut Pips,
        asset_registry: &HashMap<String, SpriteEntry>,
        living_anim: &LivingAnimLib,
        config: &ChallengeConfig,
    ) {
        self.spawned.clear();
        let bg_positions = config.bad_guy_formation.positions(config.bad_guys.len());
        let bandit = match asset_registry.get("bandit-1") {
            Some(s) => s,
            None => asset_registry
                .get("__white__")
                .expect("missing bandit-1 and __white__ sprite"),
        };
        for (i, spec) in config.bad_guys.iter().enumerate() {
            let (lat_offset, d_offset) = bg_positions[i];
            let lateral = lat_offset;
            let d = DUEL_SPAWN_DISTANCE + d_offset;
            let pip = pips.make(living_roller_bundle(
                lateral,
                d,
                Vec2::splat(1.28125),
                Vec4::ONE,
                bandit,
                living_anim,
                "bandit",
            ));
            if matches!(spec.kind, BadGuyKind::Offscreen { .. }) {
                if let Some(brush) = gather_mut(&pips.ids, &mut pips.tables.core.brushes, pip) {
                    brush.color.w = 0.0;
                }
            }
            self.spawned.push((pip, spec.clone()));
        }

        self.spawned_townspeople.clear();
        let tp_positions = config.townsperson_formation.positions(config.townspeople.len());
        let townie = match asset_registry.get("townie_1") {
            Some(s) => s,
            None => asset_registry
                .get("__white__")
                .expect("missing townie_1 and __white__ sprite"),
        };
        for (i, spec) in config.townspeople.iter().enumerate() {
            let (_lat_offset, d_offset) = tp_positions[i];
            let lateral = match spec.side {
                Side::Left => -4.5,
                Side::Right => 4.5,
            };
            let d = DUEL_SPAWN_DISTANCE + d_offset * 1.7;
            let pip = pips.make(living_roller_bundle(
                lateral,
                d,
                Vec2::splat(1.28125),
                Vec4::ONE,
                townie,
                living_anim,
                "townie",
            ));
            if matches!(spec.kind, TownspersonKind::Offscreen { .. }) {
                if let Some(brush) = gather_mut(&pips.ids, &mut pips.tables.core.brushes, pip) {
                    brush.color.w = 0.0;
                }
            }
            self.spawned_townspeople.push((pip, spec.clone()));
        }
    }

    fn check_trigger(&self, ids: &Ids, roller_depths: &Class<RollerDepth>) -> bool {
        for (pip, _) in &self.spawned {
            if let Some(depth) = gather_ref(ids, roller_depths, *pip) {
                if depth.d <= DUEL_TRIGGER_DISTANCE {
                    return true;
                }
            }
        }
        for (pip, _) in &self.spawned_townspeople {
            if let Some(depth) = gather_ref(ids, roller_depths, *pip) {
                if depth.d <= DUEL_TRIGGER_DISTANCE {
                    return true;
                }
            }
        }
        false
    }

    fn update_reticles(&mut self, pips: &mut Pips, dt: f32, slow: AnimId, fast: AnimId) {
        let Some(player) = self.player else {
            return;
        };
        let player_pos = gather_ref(&pips.ids, &pips.tables.core.xforms, player)
            .map(|t| Vec2::new(t.xyz.x, t.xyz.y))
            .unwrap_or(Vec2::ZERO);

        self.reticle_homing(player_pos, dt);
        self.reticle_separation(dt);
        self.reticle_sync(pips, player_pos, slow, fast);
    }

    fn reticle_homing(&mut self, player_pos: Vec2, dt: f32) {
        for ret in &mut self.duel.reticles {
            if ret.snapped {
                ret.lateral = player_pos.x;
                ret.d = player_pos.y;
                continue;
            }
            let pos = Vec2::new(ret.lateral, ret.d);
            let dist = pos.distance(player_pos);
            let target = if dist < RETICLE_HOMING_RANGE {
                player_pos
            } else {
                player_pos + sway_offset(ret.sway_phase + ret.speed)
            };
            let speed_mul = if dist < RETICLE_HOMING_RANGE {
                RETICLE_HOMING_SPEED_MUL
            } else {
                1.0
            };
            let new_pos = pos + aniso_step(target - pos, dt * speed_mul);
            ret.sway_phase += dt;
            ret.lateral = new_pos.x;
            ret.d = new_pos.y;
        }
    }

    fn reticle_separation(&mut self, dt: f32) {
        let positions: Vec<(usize, Vec2)> = self
            .duel
            .reticles
            .iter()
            .enumerate()
            .map(|(i, r)| (i, Vec2::new(r.lateral, r.d)))
            .collect();
        for (i, ret) in self.duel.reticles.iter_mut().enumerate() {
            if ret.snapped {
                continue;
            }
            let pos = Vec2::new(ret.lateral, ret.d);
            let mut push = Vec2::ZERO;
            for (j, other) in &positions {
                if i == *j {
                    continue;
                }
                let delta = pos - *other;
                let dist = delta.length();
                if dist < RETICLE_SEPARATION_MIN && dist > 1e-6 {
                    let strength = (RETICLE_SEPARATION_MIN - dist) / RETICLE_SEPARATION_MIN;
                    push += delta.normalize() * strength;
                }
            }
            let new_pos = pos + push * RETICLE_SEPARATION_SPEED * dt;
            ret.lateral = new_pos.x;
            ret.d = new_pos.y;
        }
    }

    fn reticle_sync(&mut self, pips: &mut Pips, player_pos: Vec2, slow: AnimId, fast: AnimId) {
        let Some(anim) = AnimWorld::tables(&mut pips.tables.pile) else {
            return;
        };
        for ret in &mut self.duel.reticles {
            let pos = Vec2::new(ret.lateral, ret.d);
            let is_fast = ret.snapped || pos.distance(player_pos) < RETICLE_HOMING_RANGE;
            if is_fast != ret.was_fast {
                ret.was_fast = is_fast;
                let new_id = if is_fast { fast } else { slow };
                if let Some(keyframe) =
                    gather_mut(&pips.ids, &mut anim.anim_keyframes, ret.pip)
                {
                    keyframe.id = new_id;
                }
                if let Some(time) = gather_mut(&pips.ids, &mut anim.anim_times, ret.pip) {
                    time.0 = f32::NAN;
                }
            }
            if let Some(x) = gather_mut(&pips.ids, &mut pips.tables.core.xforms, ret.pip) {
                x.xyz.x = pos.x;
                x.xyz.y = pos.y;
                x.xyz.z = 0.2;
            }
        }
    }
}

impl Solver for DungeonMaster {}

impl DungeonMaster {
    pub fn update(
        &mut self,
        dt: f32,
        pips: &mut Pips,
        _scripts: &mut ScriptsMap,
        signals: &mut SignalsMap,
        _input: &Input,
        asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        self.duel.tick(dt);

        let Some(living_anim) = self.living_anim.clone() else {
            return;
        };

        let ids = &pips.ids;
        let Some(roller) = RollerWorld::tables(&mut pips.tables.pile) else {
            return;
        };

        if self.phase == DmPhase::WaitingToSpawn && self.challenge_index == 0 {
            if let Some(retry) = self.retry.take() {
                self.challenge_index = retry.challenge_index;
                if let Some(player) = self.player {
                    if let Some(depth) = gather_mut(ids, &mut roller.roller_depths, player) {
                        depth.d = retry.player_d;
                    }
                }
                log::info!(
                    "retry: resuming at challenge {} depth {:.1}",
                    retry.challenge_index,
                    retry.player_d
                );
            }
        }

        match self.phase {
            DmPhase::WaitingToSpawn => {
                if self.challenge_index == 0 {
                    self.howdy_perfect_run = HowdyPerfectRun::new();
                }
                if self.challenge_index >= self.challenges.len() {
                    self.phase = DmPhase::Finished;
                    let perfect = self.howdy_perfect_run.0;
                    self.game_stats.complete = 1;
                    if perfect {
                        self.game_stats.challenge = 1;
                    }
                    self.game_stats.highscore =
                        self.game_stats.highscore.max(self.game_stats.items_collected);
                    self.game_stats.flush();
                } else {
                    let config = self.challenges[self.challenge_index].clone();
                    self.spawn_challenge(pips, asset_registry, &living_anim, &config);
                    self.phase = DmPhase::ChallengeActive;
                }
            }
            DmPhase::ChallengeActive => {
                if self.duel.phase == DuelPhase::Idle {
                    if self.check_trigger(ids, &roller.roller_depths) {
                        let pending = PendingDuel {
                            bad_guys: std::mem::take(&mut self.spawned),
                            townspeople: std::mem::take(&mut self.spawned_townspeople),
                        };
                        self.duel.request(pending);
                        self.phase = DmPhase::DuelInProgress;
                    }
                }
            }
            DmPhase::DuelInProgress => {
                if self.duel.phase == DuelPhase::Idle {
                    for ret in self.duel.reticles.drain(..) {
                        pips.destroy(ret.pip);
                    }
                    self.game_stats.completed_levels += 1;
                    self.phase = DmPhase::PostDuelDelay;
                    self.delay_timer = 0.0;
                } else {
                    let (lib, slow, fast) = match self.reticle_anim.as_ref() {
                        Some(a) => (Some(a.lib), Some(a.slow_anim), Some(a.fast_anim)),
                        None => (None, None, None),
                    };
                    if self.duel.phase == DuelPhase::Active && self.duel.reticles.is_empty() {
                        if let (Some(lib), Some(slow), _) = (lib, slow, fast) {
                            let new = spawn_reticles_for(
                                &self.duel.bad_guys,
                                pips,
                                asset_registry,
                                lib,
                                slow,
                            );
                            self.duel.reticles.extend(new);
                        }
                    }
                    if let (Some(slow), Some(fast)) = (slow, fast) {
                        self.update_reticles(pips, dt, slow, fast);
                    }
                }
            }
            DmPhase::PostDuelDelay => {
                self.delay_timer += dt;
                if self.delay_timer >= POST_DUEL_DELAY {
                    self.challenge_index += 1;
                    self.phase = DmPhase::WaitingToSpawn;
                }
            }
            DmPhase::Finished => {}
        }

        if let Some(duel_signals) = DuelWorld::signals(signals) {
            duel_signals.duel_state = self.duel.state();
        }
    }
}

fn spawn_reticles_for(
    bad_guys: &[(PipId, BadGuySpec)],
    pips: &mut Pips,
    asset_registry: &HashMap<String, SpriteEntry>,
    lib: AnimLibId,
    slow: AnimId,
) -> Vec<DuelReticle> {
    let bad_aim = asset_registry
        .get("bad_aim")
        .unwrap_or(asset_registry.get("__white__").expect("missing __white__ sprite"));
    let size = Vec2::splat(0.5);
    let color = Vec4::new(180.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 1.0);
    bad_guys
        .iter()
        .enumerate()
        .map(|(i, (pip, _spec))| {
            let pos = gather_ref(&pips.ids, &pips.tables.core.xforms, *pip)
                .map(|t| t.xyz)
                .unwrap_or(Vec3::ZERO);
            let seed = (i as f32) * 1.5;
            let ret = pips.make(duel_reticle_bundle(
                pos, size, color, bad_aim, lib, slow, "reticle",
            ));
            DuelReticle {
                pip: ret,
                lateral: pos.x,
                d: pos.y,
                speed: seed,
                sway_phase: 0.0,
                snapped: false,
                was_fast: false,
            }
        })
        .collect()
}

const BADGUY_WANDER_SPEED: f32 = 0.4;
const BADGUY_WANDER_AMP: f32 = 0.65;
const RETICLE_SPEED_X: f32 = 1.5;
const RETICLE_SPEED_Y: f32 = 0.9;
const RETICLE_SNAP_THRESHOLD: f32 = 0.60;
const RETICLE_HOMING_RANGE: f32 = 0.95;
const RETICLE_HOMING_SPEED_MUL: f32 = 3.8;
const SWAY_RADIUS: f32 = 1.4;
const SWAY_SPEED: f32 = 2.1;
const RETICLE_SEPARATION_MIN: f32 = 2.9;
const RETICLE_SEPARATION_SPEED: f32 = 3.0;
const ARENA_MIN_Y: f32 = -4.5;
const ARENA_MAX_Y: f32 = 0.0;

fn aniso_step(delta: Vec2, dt: f32) -> Vec2 {
    let scaled = Vec2::new(delta.x / RETICLE_SPEED_X, delta.y / RETICLE_SPEED_Y);
    let scaled_len = scaled.length();
    if scaled_len < 1e-6 {
        return Vec2::ZERO;
    }
    let step = dt.min(scaled_len);
    let unit = scaled / scaled_len;
    Vec2::new(unit.x * RETICLE_SPEED_X, unit.y * RETICLE_SPEED_Y) * step
}

fn sway_offset(time: f32) -> Vec2 {
    Vec2::new(
        SWAY_RADIUS * (SWAY_SPEED * time).sin(),
        SWAY_RADIUS * (SWAY_SPEED * 2.0 * time).sin(),
    )
}
