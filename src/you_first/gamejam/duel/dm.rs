use glam::{Vec2, Vec4};

use crate::addition::Addition;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::ecs::PipId;
use crate::ecs::core::CoreTablesWorld;
use crate::you_first::gamejam::duel::formation::Formation;
use crate::you_first::gamejam::duel::state::{
    BadGuyKind, BadGuySpec, ChallengeConfig, Duel, DuelState, HowdyPerfectRun, LivingAnimLib,
    PendingDuel, ReticlePattern, RetryState, Side, TownspersonKind, TownspersonSpec,
};
use crate::you_first::gamejam::roller::bundles::living_roller_bundle;
use crate::you_first::gamejam::roller::components::RollerWorld;
use crate::you_first::gamejam::roller::projection::{DUEL_SPAWN_DISTANCE, DUEL_TRIGGER_DISTANCE};
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
    pub player: PipId,
    pub living_anim: LivingAnimLib,
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
        player: PipId,
        living_anim: LivingAnimLib,
        retry: Option<RetryState>,
    ) -> Self {
        Self {
            player,
            living_anim,
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

    fn spawn_challenge(&mut self, ctx: &mut DomainView, config: &ChallengeConfig) {
        self.spawned.clear();
        let bg_positions = config.bad_guy_formation.positions(config.bad_guys.len());
        let bandit = match ctx.asset_registry.get("bandit-1") {
            Some(s) => s,
            None => ctx
                .asset_registry
                .get("__white__")
                .expect("missing bandit-1 and __white__ sprite"),
        };
        for (i, spec) in config.bad_guys.iter().enumerate() {
            let (lat_offset, d_offset) = bg_positions[i];
            let lateral = lat_offset;
            let d = DUEL_SPAWN_DISTANCE + d_offset;
            let pip = ctx.domain.make(living_roller_bundle(
                lateral,
                d,
                Vec2::splat(1.28125),
                Vec4::ONE,
                bandit,
                &self.living_anim,
                "bandit",
            ));
            if matches!(spec.kind, BadGuyKind::Offscreen { .. }) {
                //todo this is supposed to be gather
                if let Some(ptr) = ctx.domain.ids.get(pip)
                    && let Some(core) = CoreTablesWorld::tables(&mut ctx.domain.tables)
                    && let Some(brush) = core.brushes.get_row_mut(ptr)
                {
                    brush.color.w = 0.0;
                }
            }
            self.spawned.push((pip, spec.clone()));
        }

        self.spawned_townspeople.clear();
        let tp_positions = config.townsperson_formation.positions(config.townspeople.len());
        let townie = match ctx.asset_registry.get("townie_1") {
            Some(s) => s,
            None => ctx
                .asset_registry
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
            let pip = ctx.domain.make(living_roller_bundle(
                lateral,
                d,
                Vec2::splat(1.28125),
                Vec4::ONE,
                townie,
                &self.living_anim,
                "townie",
            ));
            if matches!(spec.kind, TownspersonKind::Offscreen { .. }) {
                //gather...
                if let Some(ptr) = ctx.domain.ids.get(pip)
                    && let Some(core) = CoreTablesWorld::tables(&mut ctx.domain.tables)
                    && let Some(brush) = core.brushes.get_row_mut(ptr)
                {
                    brush.color.w = 0.0;
                }
            }
            self.spawned_townspeople.push((pip, spec.clone()));
        }
    }

    fn check_trigger(&self, ctx: &DomainView) -> bool {
        let Some(roller) = RollerWorld::tables(&mut ctx.domain.tables) else {
            return false;
        };
        for (pip, _) in &self.spawned {
            if let Some(ptr) = ctx.domain.ids.get(*pip) {
                if let Some(depth) = roller.roller_depths.get_row(ptr) {
                    if depth.d <= DUEL_TRIGGER_DISTANCE {
                        return true;
                    }
                }
            }
        }
        for (pip, _) in &self.spawned_townspeople {
            if let Some(ptr) = ctx.domain.ids.get(*pip) {
                if let Some(depth) = roller.roller_depths.get_row(ptr) {
                    if depth.d <= DUEL_TRIGGER_DISTANCE {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Script for DungeonMaster {
    fn update(&mut self, ctx: &mut DomainView) {
        self.duel.tick(ctx.dt);

        if self.phase == DmPhase::WaitingToSpawn && self.challenge_index == 0 {
            if let Some(retry) = self.retry.take() {
                self.challenge_index = retry.challenge_index;
                if let Some(roller) = RollerWorld::tables(&mut ctx.domain.tables) {
                    if let Some(ptr) = ctx.domain.ids.get(self.player)
                        && let Some(depth) = roller.roller_depths.get_row_mut(ptr)
                    {
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
                    return;
                }
                let config = self.challenges[self.challenge_index].clone();
                self.spawn_challenge(ctx, &config);
                self.phase = DmPhase::ChallengeActive;
            }
            DmPhase::ChallengeActive => {
                if !matches!(self.duel.state, DuelState::Inactive) {
                    return;
                }
                if self.check_trigger(ctx) {
                    let pending = PendingDuel {
                        bad_guys: std::mem::take(&mut self.spawned),
                        townspeople: std::mem::take(&mut self.spawned_townspeople),
                    };
                    self.duel.request(pending);
                    self.phase = DmPhase::DuelInProgress;
                }
            }
            DmPhase::DuelInProgress => {
                if matches!(self.duel.state, DuelState::Inactive) {
                    self.game_stats.completed_levels += 1;
                    self.phase = DmPhase::PostDuelDelay;
                    self.delay_timer = 0.0;
                }
            }
            DmPhase::PostDuelDelay => {
                self.delay_timer += ctx.dt;
                if self.delay_timer >= POST_DUEL_DELAY {
                    self.challenge_index += 1;
                    self.phase = DmPhase::WaitingToSpawn;
                }
            }
            DmPhase::Finished => {}
        }
    }
}
