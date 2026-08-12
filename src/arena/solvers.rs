use std::collections::HashMap;

use glam::{Quat, Vec2, Vec3};
use rand::Rng;

use crate::arena::makers::{ActorBlueprint, PickupBlueprint};
use crate::arena::tables::{
    ActorAddition, ArenaAddition, HealthData, HealthPickupData, PilotData, PilotState, Team,
};
use crate::brushes::Brush;
use crate::gather::impls::gather_mut;
use crate::query::impls::{query_mut_mut, query_mut_mut_mut, query_ref_ref};
use crate::scripting::{DomainView, Script};
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::tables::PipId;
use crate::tables::tables::{AdditionsView, Tables};

const BOUNDS: f32 = 15.0;
const HIT_RADIUS: f32 = 0.3;
const PICKUP_RADIUS: f32 = 0.8;

pub struct MovementSolver;

impl Script for MovementSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt();
        let domain = ctx.domain();
        for (motions, xforms) in query_mut_mut(
            &mut domain.tables.core.motions,
            &(),
            &mut domain.tables.core.xforms,
            &(),
        ) {
            for (motion, xform) in motions.iter_mut().zip(xforms.iter_mut()) {
                xform.xyz += motion.vel * dt;
            }
        }
    }
}

pub struct BoundsSolver;

impl Script for BoundsSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let domain = ctx.domain();
        for (motions, xforms) in query_mut_mut(
            &mut domain.tables.core.motions,
            &(),
            &mut domain.tables.core.xforms,
            &(),
        ) {
            for (motion, xform) in motions.iter_mut().zip(xforms.iter_mut()) {
                for axis in 0..2 {
                    if xform.xyz[axis] > BOUNDS {
                        xform.xyz[axis] = BOUNDS;
                        motion.vel[axis] = -motion.vel[axis];
                    } else if xform.xyz[axis] < -BOUNDS {
                        xform.xyz[axis] = -BOUNDS;
                        motion.vel[axis] = -motion.vel[axis];
                    }
                }
                xform.xyz.z = 0.0;
            }
        }
    }
}

pub struct ProjectileSolver;

impl ProjectileSolver {
    fn team_map(tables: &Tables) -> HashMap<PipId, Team> {
        let mut map = HashMap::new();
        let Some(actor) = tables.get::<ActorAddition>() else {
            return map;
        };
        for (team_col, pip_col) in query_ref_ref(&actor.team, &(), &tables.system.pip_id, &()) {
            for i in 0..team_col.len() {
                map.insert(pip_col[i], team_col[i]);
            }
        }
        map
    }

    fn additions<'a>(additions: &'a mut AdditionsView<'a>) -> &'a mut ArenaAddition {
        additions
            .get_mut::<ArenaAddition>()
            .expect("arena addition not registered")
    }
}

impl Script for ProjectileSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt();
        let yellow = *ctx
            .asset_registry()
            .try_get("townie_1")
            .unwrap_or(ctx.asset_registry().get("yellow"));
        let mut pickup_brush = Brush::from_sprite(&yellow);
        pickup_brush.scale *= 1.5;

        let mut to_destroy: Vec<PipId> = Vec::new();
        let mut to_spawn: Vec<PickupBlueprint> = Vec::new();

        {
            let (_, tables) = ctx.split();
            let team_map = Self::team_map(tables);
            let mut view = tables.view();
            let xforms = &view.core.xforms;
            let system = &view.system;
            let arena = Self::additions(&mut view.additions);

            for (class_id, projectile_col) in arena.projectile.columns_mut() {
                let Some(xform_col) = xforms.get_col(class_id) else {
                    continue;
                };
                let Some(pip_col) = system.pip_id.get_col(class_id) else {
                    continue;
                };

                for i in 0..projectile_col.len() {
                    let data = &mut projectile_col[i];
                    data.lifetime -= dt;
                    if data.lifetime <= 0.0 {
                        to_destroy.push(pip_col[i]);
                        continue;
                    }
                    let pos = xform_col[i].xyz.truncate();
                    let damage = data.damage;

                    'hit: for (health_class_id, health_col) in arena.health.columns_mut() {
                        let Some(xform_h) = xforms.get_col(health_class_id) else {
                            continue;
                        };
                        let Some(pip_h) = system.pip_id.get_col(health_class_id) else {
                            continue;
                        };

                        for j in 0..health_col.len() {
                            if health_col[j].health <= 0.0 {
                                continue;
                            }
                            if (xform_h[j].xyz.truncate() - pos).length() >= HIT_RADIUS {
                                continue;
                            }
                            health_col[j].health -= damage;
                            to_destroy.push(pip_col[i]);
                            to_destroy.push(pip_h[j]);
                            if team_map.get(&pip_h[j]) == Some(&Team::Enemy)
                                && health_col[j].health <= 0.0
                            {
                                to_spawn.push(PickupBlueprint {
                                    xform: xform_h[j].clone(),
                                    brush: pickup_brush.clone(),
                                    name: None,
                                    team: Some(Team::Pickup),
                                    pickup: HealthPickupData { amount: 15.0 },
                                });
                            }
                            break 'hit;
                        }
                    }
                }
            }
        }

        let domain = ctx.domain();
        for id in to_destroy.drain(..) {
            domain.destroy(id);
        }
        for pickup in to_spawn.drain(..) {
            domain.make(pickup);
        }
    }
}

pub struct SpawnerSolver {
    pub player: Option<PipId>,
}

impl SpawnerSolver {
    fn make_enemy(&self, pos: Vec2, brush: Brush) -> ActorBlueprint {
        let pilot = match self.player {
            Some(p) => PilotData {
                state: PilotState::Chase { target: p },
                speed: 2.0,
                cooldown: 0.0,
            },
            None => PilotData {
                state: PilotState::Wander {
                    goal: pos,
                    timer: 2.0,
                },
                speed: 2.0,
                cooldown: 0.0,
            },
        };
        ActorBlueprint {
            xform: Transform {
                xyz: Vec3::new(pos.x, pos.y, 0.0),
                rot: Quat::IDENTITY,
            },
            brush,
            name: None,
            motion: Some(Motion { vel: Vec3::ZERO }),
            team: Some(Team::Enemy),
            health: Some(HealthData {
                health: 30.0,
                max: 30.0,
            }),
            pilot: Some(pilot),
        }
    }
}

impl Script for SpawnerSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let dt = ctx.dt();

        let mut to_spawn: Vec<ActorBlueprint> = Vec::new();

        {
            let (_, tables) = ctx.split();
            let mut view = tables.view();
            let arena = view.additions.get_mut::<ArenaAddition>().unwrap();
            let mut rng = rand::rng();

            for (spawner_col, xform_col) in
                query_mut_mut(&mut arena.spawner, &(), &mut view.core.xforms, &())
            {
                for i in 0..spawner_col.len() {
                    let data = &mut spawner_col[i];
                    let pos = xform_col[i].xyz.truncate();
                    data.timer -= dt;
                    if data.timer > 0.0 || data.spawned >= data.max_count {
                        continue;
                    }
                    let offset =
                        Vec2::new(rng.random::<f32>() - 0.5, rng.random::<f32>() - 0.5) * 2.0;
                    let brush = data.enemy_brush.clone();
                    to_spawn.push(self.make_enemy(pos + offset, brush));
                    data.spawned += 1;
                    data.timer = data.interval;
                }
            }
        }

        let domain = ctx.domain();
        for actor in to_spawn.drain(..) {
            domain.make(actor);
        }
    }
}

pub struct PickupSolver {
    pub player: Option<PipId>,
}

impl PickupSolver {
    fn additions<'a>(additions: &'a mut AdditionsView<'a>) -> &'a mut ArenaAddition {
        additions
            .get_mut::<ArenaAddition>()
            .expect("arena addition not registered")
    }
}

impl Script for PickupSolver {
    fn update(&mut self, ctx: &mut DomainView) {
        let Some(player) = self.player else { return };

        let mut to_destroy: Vec<PipId> = Vec::new();
        let mut to_heal: Vec<(PipId, f32)> = Vec::new();

        {
            let (ids, tables) = ctx.split();
            let mut view = tables.view();
            let arena = Self::additions(&mut view.additions);

            let player_pos = if let Some(ptr) = ids.get(player) {
                view.core.xforms
                    .get_row(ptr)
                    .map_or(Vec2::ZERO, |x| x.xyz.truncate())
            } else {
                Vec2::ZERO
            };

            for (pickup_col, xform_col, pip_col) in query_mut_mut_mut(
                &mut arena.pickup,
                &(),
                &mut view.core.xforms,
                &(),
                &mut view.system.pip_id,
                &(),
            ) {
                for i in 0..pickup_col.len() {
                    let pos = xform_col[i].xyz.truncate();
                    if (pos - player_pos).length() < PICKUP_RADIUS {
                        to_heal.push((player, pickup_col[i].amount));
                        to_destroy.push(pip_col[i]);
                    }
                }
            }

            for (pip, amount) in to_heal.drain(..) {
                if let Some(h) = gather_mut(ids, &mut arena.health, pip) {
                    h.health = (h.health + amount).min(h.max);
                }
            }
        }

        let domain = ctx.domain();
        for pip in to_destroy.drain(..) {
            domain.destroy(pip);
        }
    }
}
