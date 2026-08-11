use std::any::{Any, TypeId};
use std::collections::HashMap;

use glam::{Quat, Vec2, Vec3};
use rand::Rng;

use crate::arena::makers::ProjectileBlueprint;
use crate::arena::tables::{PilotAddition, PilotState, ProjectileData, Team, TeamAddition};
use crate::brushes::Brush;
use crate::gather::impls::gather_ref;
use crate::scripting::{DomainView, Script};
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;

const PROJECTILE_SPEED: f32 = 8.0;
const PROJECTILE_LIFETIME: f32 = 1.2;
const PROJECTILE_DAMAGE: f32 = 8.0;
const FIRE_COOLDOWN: f32 = 1.0;
const WANDER_BOUNDS: f32 = 10.0;

pub struct PilotScript;

impl Script for PilotScript {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt();
        let red = *ctx.asset_registry().get("red");

        let mut shots: Vec<ProjectileBlueprint> = Vec::new();

        {
            let (ids, tables) = ctx.split();
            let xforms = &tables.core.xforms;
            let motions = &mut tables.core.motions;
            let system = &tables.system;

            let (team_map, enemy_positions) = {
                let maybe_team = tables.additions.get(&TypeId::of::<TeamAddition>()).and_then(|b| {
                    let any: &dyn Any = b.as_ref();
                    any.downcast_ref::<TeamAddition>()
                });
                match maybe_team {
                    Some(team_add) => {
                        let mut teams = HashMap::new();
                        let mut enemies = Vec::new();
                        for (class_id, team_col) in team_add.team.columns() {
                            let Some(xform_col) = xforms.get_col(class_id) else { continue };
                            let Some(pip_id_col) = system.pip_id.get_col(class_id) else { continue };

                            for i in 0..team_col.len() {
                                teams.insert(pip_id_col[i], team_col[i]);
                                if team_col[i] == Team::Enemy {
                                    enemies.push(xform_col[i].xyz.truncate());
                                }
                            }
                        }
                        (teams, enemies)
                    }
                    None => (HashMap::new(), Vec::new()),
                }
            };

            let pilot = {
                let Some(boxed) = tables.additions.get_mut(&TypeId::of::<PilotAddition>()) else {
                    return;
                };
                let any: &mut dyn Any = boxed.as_mut();
                any.downcast_mut::<PilotAddition>().unwrap()
            };
            let mut rng = rand::rng();

            for (class_id, pilot_col) in pilot.data.columns_mut() {
                let Some(motion_col) = motions.get_col_mut(class_id) else { continue };
                let Some(pip_id_col) = system.pip_id.get_col(class_id) else { continue };

                for i in 0..pilot_col.len() {
                    let pilot_data = &mut pilot_col[i];
                    let motion = &mut motion_col[i];
                    let pip = pip_id_col[i];

                    let Some(xform) = gather_ref(ids, xforms, pip) else { continue };
                    let pos = xform.xyz.truncate();
                    pilot_data.cooldown -= dt;

                    let is_player = team_map.get(&pip) == Some(&Team::Player);

                    match &mut pilot_data.state {
                        PilotState::Wander { goal, timer } => {
                            *timer -= dt;
                            let to_goal = *goal - pos;

                            if is_player && !enemy_positions.is_empty() {
                                let threat = enemy_positions
                                    .iter()
                                    .fold(Vec2::ZERO, |acc, &e| acc + (pos - e).normalize_or_zero());
                                let mut dir = -threat.normalize_or_zero();
                                if dir == Vec2::ZERO {
                                    dir = to_goal.normalize_or_zero();
                                }
                                if dir == Vec2::ZERO {
                                    dir = Vec2::X;
                                }
                                *goal = pos + dir * 8.0;
                                *timer = 0.3;
                                motion.vel = Vec3::new(dir.x, dir.y, 0.0) * pilot_data.speed;
                            } else if to_goal.length() < 0.5 || *timer <= 0.0 {
                                *goal = Vec2::new(
                                    rng.random::<f32>() * WANDER_BOUNDS * 2.0 - WANDER_BOUNDS,
                                    rng.random::<f32>() * WANDER_BOUNDS * 2.0 - WANDER_BOUNDS,
                                );
                                *timer = 3.0;
                                let dir = (*goal - pos).normalize_or_zero();
                                motion.vel = Vec3::new(dir.x, dir.y, 0.0) * pilot_data.speed;
                            } else {
                                let dir = to_goal.normalize_or_zero();
                                motion.vel = Vec3::new(dir.x, dir.y, 0.0) * pilot_data.speed;
                            }
                        }
                        PilotState::Chase { target } => {
                            let Some(target_xform) = gather_ref(ids, xforms, *target) else {
                                pilot_data.state = PilotState::Wander { goal: pos, timer: 1.0 };
                                continue;
                            };
                            let target_pos = target_xform.xyz.truncate();
                            let dir = (target_pos - pos).normalize_or_zero();
                            motion.vel = Vec3::new(dir.x, dir.y, 0.0) * pilot_data.speed;

                            if pilot_data.cooldown <= 0.0 && dir != Vec2::ZERO {
                                let spawn_pos = pos + dir * 0.8;
                                shots.push(ProjectileBlueprint {
                                    xform: Transform {
                                        xyz: Vec3::new(spawn_pos.x, spawn_pos.y, 0.0),
                                        rot: Quat::IDENTITY,
                                    },
                                    brush: Brush {
                                        canvas: red.canvas,
                                        material: red.material,
                                        scale: red.natural_scale * 0.8,
                                        color: Vec3::ONE.extend(1.0),
                                    },
                                    motion: Motion { vel: Vec3::new(dir.x, dir.y, 0.0) * PROJECTILE_SPEED },
                                    projectile: ProjectileData {
                                        lifetime: PROJECTILE_LIFETIME,
                                        damage: PROJECTILE_DAMAGE,
                                        owner: pip,
                                    },
                                });
                                pilot_data.cooldown = FIRE_COOLDOWN;
                            }
                        }
                    }
                }
            }
        }

        let domain = ctx.domain();
        for shot in shots.drain(..) {
            domain.make(shot);
        }
    }
}
