use glam::Vec2;

use crate::gamejam::duel::state::{LivingAnimLib, TumbleweedAnimLib};
use crate::gamejam::roller::bundle::{LivingRollerBundle, RollerSpriteBundle, TumbleweedRollerBundle};
use crate::gamejam::roller::projection::{FAR_Z, NEAR_Z};
use crate::scenes::SceneResources;
use crate::world::World;

#[derive(Debug, Clone, Copy)]
pub enum Placement {
    Alternate,
    Random,
    Fixed(bool),
    SideRandom { min_offset: f32 },
}

#[derive(Debug, Clone, Copy)]
pub enum FlipMode {
    None,
    BySide,
    Random,
}

#[derive(Debug, Clone, Copy)]
pub struct ScatterMotion {
    pub spawn_depth: f32,
    pub lateral_speed: f32,
    pub pre_seed: bool,
}

impl Default for ScatterMotion {
    fn default() -> Self {
        Self {
            spawn_depth: FAR_Z,
            lateral_speed: 0.0,
            pre_seed: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimKind {
    None,
    Living,
    Tumbleweed,
}

pub struct ScatterChannel {
    material: &'static str,
    interval: f32,
    lateral_range: f32,
    size: Vec2,
    color: [u8; 4],
    scalar: f32,
    placement: Placement,
    flip_mode: FlipMode,
    anim_kind: AnimKind,
    motion: ScatterMotion,
    next_spawn: f32,
    left_side: bool,
}

impl ScatterChannel {
    fn place(&mut self, rng: &mut u32) -> (f32, bool) {
        let (offset, is_left) = match self.placement {
            Placement::Alternate => {
                let is_left = self.left_side;
                let o = if is_left {
                    -self.lateral_range
                } else {
                    self.lateral_range
                };
                self.left_side = !self.left_side;
                (o, is_left)
            }
            Placement::Random => {
                let r = next_random(rng) * 2.0 - 1.0;
                (r * self.lateral_range, r < 0.0)
            }
            Placement::Fixed(is_left) => {
                let o = if is_left { -self.lateral_range } else { self.lateral_range };
                (o, is_left)
            }
            Placement::SideRandom { min_offset } => {
                let is_left = next_random(rng) < 0.5;
                let r = next_random(rng);
                if is_left {
                    (-(min_offset + r * (self.lateral_range - min_offset)), true)
                } else {
                    (min_offset + r * (self.lateral_range - min_offset), false)
                }
            }
        };
        (offset, is_left)
    }

    fn spawn(&self, world: &mut World, resources: &SceneResources<'_>, living_anim: LivingAnimLib, tumbleweed_anim: TumbleweedAnimLib, lateral: f32, d: f32, is_left: bool, rng: &mut u32) {
        let is_flipped = match self.flip_mode {
            FlipMode::None => false,
            FlipMode::BySide => is_left,
            FlipMode::Random => next_random(rng) < 0.5,
        };
        match self.anim_kind {
            AnimKind::None => {
                let mut bundle = RollerSpriteBundle::from_asset(
                    lateral,
                    d,
                    self.size,
                    self.color,
                    self.material,
                    resources,
                );
                bundle.roller_depth.scalar = self.scalar;
                bundle.roller_depth.lateral_speed = self.motion.lateral_speed;
                bundle.brush_flip.is_flipped = is_flipped;
                bundle.spawn(world);
            }
            AnimKind::Living => {
                let mut bundle = LivingRollerBundle::from_asset(
                    lateral,
                    d,
                    self.size,
                    self.color,
                    self.material,
                    living_anim,
                    resources,
                );
                bundle.roller_depth.scalar = self.scalar;
                bundle.roller_depth.lateral_speed = self.motion.lateral_speed;
                bundle.brush_flip.is_flipped = is_flipped;
                bundle.spawn(world);
            }
            AnimKind::Tumbleweed => {
                let mut bundle = TumbleweedRollerBundle::from_asset(
                    lateral,
                    d,
                    self.size,
                    self.color,
                    self.material,
                    tumbleweed_anim,
                    resources,
                );
                bundle.roller_depth.scalar = self.scalar;
                bundle.roller_depth.lateral_speed = self.motion.lateral_speed;
                bundle.brush_flip.is_flipped = is_flipped;
                bundle.spawn(world);
            }
        }
    }
}

pub struct Biome {
    channels: Vec<ScatterChannel>,
}

impl Biome {
    fn pre_seed(
        &mut self,
        world: &mut World,
        resources: &SceneResources<'_>,
        living_anim: LivingAnimLib,
        tumbleweed_anim: TumbleweedAnimLib,
        rng: &mut u32,
    ) {
        for channel in &mut self.channels {
            if !channel.motion.pre_seed {
                continue;
            }

            let steps = ((FAR_Z - NEAR_Z) / 2.0) as usize;
            for i in 0..steps {
                let d = FAR_Z - (i as f32) * 2.0;
                if d < NEAR_Z {
                    break;
                }

                let (lateral, is_left) = channel.place(rng);
                channel.spawn(
                    world,
                    resources,
                    living_anim,
                    tumbleweed_anim,
                    lateral,
                    d,
                    is_left,
                    rng,
                );
            }
        }
    }

    fn update(
        &mut self,
        world: &mut World,
        resources: &SceneResources<'_>,
        living_anim: LivingAnimLib,
        tumbleweed_anim: TumbleweedAnimLib,
        walk_distance: f32,
        rng: &mut u32,
    ) {
        for channel in &mut self.channels {
            if walk_distance >= channel.next_spawn {
                channel.next_spawn = walk_distance + channel.interval;

                let d = channel.motion.spawn_depth;
                let (lateral, is_left) = channel.place(rng);
                channel.spawn(
                    world,
                    resources,
                    living_anim,
                    tumbleweed_anim,
                    lateral,
                    d,
                    is_left,
                    rng,
                );
            }
        }
    }

    pub fn sparse_desert() -> Self {
        Self {
            channels: vec![
                ScatterChannel {
                    material: "cactus",
                    interval: 3.0,
                    lateral_range: 3.5,
                    size: Vec2::new(0.6, 1.0),
                    color: [255, 255, 255, 255],
                    scalar: 1.0,
                    placement: Placement::Alternate,
                    flip_mode: FlipMode::Random,
                    anim_kind: AnimKind::Living,
                    motion: ScatterMotion::default(),
                    next_spawn: 0.0,
                    left_side: true,
                },
            ],
        }
    }

    pub fn default_desert() -> Self {
        Self {
            channels: vec![
                ScatterChannel {
                    material: "cactus",
                    interval: 2.0,
                    lateral_range: 4.0,
                    size: Vec2::new(0.6, 1.0),
                    color: [255, 255, 255, 255],
                    scalar: 1.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::Random,
                    anim_kind: AnimKind::Living,
                    motion: ScatterMotion::default(),
                    next_spawn: 0.0,
                    left_side: true,
                },
                ScatterChannel {
                    material: "tumbleweed",
                    interval: 5.0,
                    lateral_range: 5.0,
                    size: Vec2::new(0.4, 0.4),
                    color: [255, 255, 255, 255],
                    scalar: 1.0,
                    placement: Placement::Fixed(true),
                    flip_mode: FlipMode::BySide,
                    anim_kind: AnimKind::Tumbleweed,
                    motion: ScatterMotion {
                        spawn_depth: FAR_Z,
                        lateral_speed: 1.2,
                        pre_seed: false,
                    },
                    next_spawn: 0.0,
                    left_side: true,
                },
            ],
        }
    }
}

fn next_random(rng: &mut u32) -> f32 {
    *rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
    ((*rng & 0x7fff) as f32) / 32768.0
}
