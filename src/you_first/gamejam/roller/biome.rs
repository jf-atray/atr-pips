use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::anims::{AnimKeyframe, AnimRules, AnimSpin, AnimTime, AnimWorld, AnimXyz};
use crate::assets::SpriteEntry;
use crate::addition::ExampleDomain;
use crate::ecs::scope::Maker;
use crate::you_first::gamejam::duel::state::TumbleweedAnimLib;
use crate::you_first::gamejam::roller::bundles::roller_body;
use crate::you_first::gamejam::roller::components::{BrushFlip, RollerDepth};
use crate::you_first::gamejam::roller::projection::{FAR_Z, NEAR_Z};

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

#[derive(Debug)]
pub struct ScatterChannel {
    material: &'static str,
    interval: f32,
    lateral_range: f32,
    size: Vec2,
    scalar: f32,
    placement: Placement,
    flip_mode: FlipMode,
    motion: ScatterMotion,
    next_spawn: f32,
    left_side: bool,
}

impl ScatterChannel {
    fn place(&mut self) -> (f32, bool) {
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
                let r = rand::random::<f32>() * 2.0 - 1.0;
                (r * self.lateral_range, r < 0.0)
            }
            Placement::Fixed(is_left) => {
                let o = if is_left {
                    -self.lateral_range
                } else {
                    self.lateral_range
                };
                (o, is_left)
            }
            Placement::SideRandom { min_offset } => {
                let is_left = rand::random::<bool>();
                let r = rand::random::<f32>();
                if is_left {
                    (-(min_offset + r * (self.lateral_range - min_offset)), true)
                } else {
                    (min_offset + r * (self.lateral_range - min_offset), false)
                }
            }
        };
        (offset, is_left)
    }

    fn to_maker(
        &self,
        asset_registry: &HashMap<String, SpriteEntry>,
        lateral: f32,
        d: f32,
        is_left: bool,
        tumbleweed: Option<(TumbleweedAnimLib, AnimRules)>,
    ) -> impl Maker {
        let is_flipped = match self.flip_mode {
            FlipMode::None => false,
            FlipMode::BySide => is_left,
            FlipMode::Random => rand::random::<bool>(),
        };
        let sprite = asset_registry
            .get(self.material)
            .unwrap_or_else(|| asset_registry.get("__white__").unwrap());
        let canvas = sprite.canvas;
        let mat = sprite.material;
        let base_scale = self.size / sprite.natural_scale;
        let roller_depth = RollerDepth {
            d,
            lateral,
            speed: 0.0,
            scalar: self.scalar,
            lateral_speed: self.motion.lateral_speed,
            base_scale,
        };
        let brush_flip = BrushFlip { is_flipped };
        let body = roller_body(
            canvas,
            mat,
            Vec3::new(0.0, 0.0, 1.0),
            String::new(),
            roller_depth,
            brush_flip,
        );
        let is_tumbleweed = self.material == "tumbleweed";
        move |scope: &mut crate::ecs::scope::Scope| {
            body.make_into(scope);
            if is_tumbleweed && let Some((lib, rules)) = &tumbleweed {
                let av = scope.view::<AnimWorld>().unwrap();
                av.anim_times = Some(AnimTime(0.0));
                av.anim_keyframes = Some(AnimKeyframe {
                    id: lib.root_anim,
                    lib: lib.lib,
                });
                av.anim_spins = Some(AnimSpin {
                    func: rules.spin.clone(),
                    rot_base: 0.0,
                });
                av.anim_xyzs = Some(AnimXyz(rules.xyz.clone()));
            }
        }
    }
}

#[derive(Debug)]
pub struct Biome {
    channels: Vec<ScatterChannel>,
    tumbleweed: Option<(TumbleweedAnimLib, AnimRules)>,
}

impl Biome {
    pub fn pre_seed(&mut self, domain: &mut ExampleDomain, asset_registry: &HashMap<String, SpriteEntry>) {
        let tumbleweed = self.tumbleweed.clone();
        for channel in &mut self.channels {
            if !channel.motion.pre_seed {
                continue;
            }

            let step = channel.interval.max(0.5);
            let mut d = FAR_Z - step;
            while d >= NEAR_Z {
                let (lateral, is_left) = channel.place();
                domain.make(channel.to_maker(asset_registry, lateral, d, is_left, tumbleweed.clone()));
                d -= step;
            }
        }
    }

    pub fn update(
        &mut self,
        domain: &mut ExampleDomain,
        asset_registry: &HashMap<String, SpriteEntry>,
        walk_distance: f32,
    ) {
        let tumbleweed = self.tumbleweed.clone();
        for channel in &mut self.channels {
            while walk_distance >= channel.next_spawn {
                channel.next_spawn += channel.interval;

                let d = channel.motion.spawn_depth;
                let (lateral, is_left) = channel.place();
                domain.make(channel.to_maker(asset_registry, lateral, d, is_left, tumbleweed.clone()));
            }
        }
    }

    pub fn sparse_desert(tumbleweed: Option<(TumbleweedAnimLib, AnimRules)>) -> Self {
        Self {
            channels: vec![
                ScatterChannel {
                    material: "cactus",
                    interval: 0.2,
                    lateral_range: 3.5,
                    size: Vec2::new(2.0, 2.0),
                    scalar: 1.0,
                    placement: Placement::SideRandom { min_offset: 6.0 },
                    flip_mode: FlipMode::Random,
                    motion: ScatterMotion::default(),
                    next_spawn: 2.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "cactus",
                    interval: 1.9,
                    lateral_range: 6.5,
                    size: Vec2::new(2.0, 2.0),
                    scalar: 1.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::Random,
                    motion: ScatterMotion::default(),
                    next_spawn: 2.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "grass",
                    interval: 1.0,
                    lateral_range: 4.0,
                    size: Vec2::new(1.0, 1.0),
                    scalar: 0.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 1.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "grass_tiny",
                    interval: 0.05,
                    lateral_range: 10.0,
                    size: Vec2::new(0.25, 0.25),
                    scalar: 0.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 0.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "tumbleweed",
                    interval: 3.0,
                    lateral_range: 10.0,
                    size: Vec2::new(1.0, 1.0),
                    scalar: 1.0,
                    placement: Placement::Fixed(true),
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion {
                        spawn_depth: 8.0,
                        lateral_speed: 3.0,
                        pre_seed: false,
                    },
                    next_spawn: 4.0,
                    left_side: true,
                },
            ],
            tumbleweed,
        }
    }

    pub fn default_desert(tumbleweed: Option<(TumbleweedAnimLib, AnimRules)>) -> Self {
        Self {
            channels: vec![
                ScatterChannel {
                    material: "building_right",
                    interval: 2.0,
                    lateral_range: 7.5,
                    size: Vec2::new(7.28125, 5.0),
                    scalar: 1.0,
                    placement: Placement::Fixed(false),
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 5.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "building_left",
                    interval: 2.0,
                    lateral_range: 7.5,
                    size: Vec2::new(7.28125, 5.0),
                    scalar: 1.0,
                    placement: Placement::Fixed(true),
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 5.0,
                    left_side: true,
                },
                ScatterChannel {
                    material: "grass_tiny",
                    interval: 0.5,
                    lateral_range: 3.0,
                    size: Vec2::new(0.25, 0.25),
                    scalar: 0.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 0.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "cactus",
                    interval: 2.8,
                    lateral_range: 6.5,
                    size: Vec2::new(2.0, 2.0),
                    scalar: 1.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::Random,
                    motion: ScatterMotion::default(),
                    next_spawn: 2.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "grass",
                    interval: 1.0,
                    lateral_range: 4.0,
                    size: Vec2::new(1.0, 1.0),
                    scalar: 0.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 1.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "crate",
                    interval: 3.3,
                    lateral_range: 5.0,
                    size: Vec2::new(1.0, 1.0),
                    scalar: 1.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 3.0,
                    left_side: false,
                },
                ScatterChannel {
                    material: "world_rect",
                    interval: 0.02,
                    lateral_range: 4.0,
                    size: Vec2::new(0.5, 0.2),
                    scalar: 0.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion::default(),
                    next_spawn: 0.02,
                    left_side: false,
                },
                ScatterChannel {
                    material: "tumbleweed",
                    interval: 4.7,
                    lateral_range: 10.0,
                    size: Vec2::new(1.0, 1.0),
                    scalar: 1.0,
                    placement: Placement::Fixed(true),
                    flip_mode: FlipMode::None,
                    motion: ScatterMotion {
                        spawn_depth: 8.0,
                        lateral_speed: 3.0,
                        pre_seed: false,
                    },
                    next_spawn: 4.0,
                    left_side: true,
                },
            ],
            tumbleweed,
        }
    }
}
