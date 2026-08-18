use crate::assets::AssetRegistry;
use crate::tables::domain::Domain;
use crate::you_first::gamejam::roller::bundles::roller_sprite_bundle_from_asset;
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

pub struct ScatterChannel {
    material: &'static str,
    interval: f32,
    lateral_range: f32,
    color: [u8; 4],
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

    fn spawn(
        &self,
        domain: &mut Domain,
        asset_registry: &AssetRegistry,
        lateral: f32,
        d: f32,
        is_left: bool,
    ) {
        let is_flipped = match self.flip_mode {
            FlipMode::None => false,
            FlipMode::BySide => is_left,
            FlipMode::Random => rand::random::<bool>(),
        };
        domain.make(roller_sprite_bundle_from_asset(
            self.material,
            asset_registry,
            lateral,
            d,
            self.color,
            self.scalar,
            self.motion.lateral_speed,
            is_flipped,
        ));
    }
}

pub struct Biome {
    channels: Vec<ScatterChannel>,
}

impl Biome {
    pub fn pre_seed(&mut self, domain: &mut Domain, asset_registry: &AssetRegistry) {
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

                let (lateral, is_left) = channel.place();
                channel.spawn(domain, asset_registry, lateral, d, is_left);
            }
        }
    }

    pub fn update(
        &mut self,
        domain: &mut Domain,
        asset_registry: &AssetRegistry,
        walk_distance: f32,
    ) {
        for channel in &mut self.channels {
            if walk_distance >= channel.next_spawn {
                channel.next_spawn = walk_distance + channel.interval;

                let d = channel.motion.spawn_depth;
                let (lateral, is_left) = channel.place();
                channel.spawn(domain, asset_registry, lateral, d, is_left);
            }
        }
    }

    pub fn sparse_desert() -> Self {
        Self {
            channels: vec![ScatterChannel {
                material: "cactus",
                interval: 3.0,
                lateral_range: 3.5,
                color: [255, 255, 255, 255],
                scalar: 1.0,
                placement: Placement::Alternate,
                flip_mode: FlipMode::Random,
                motion: ScatterMotion::default(),
                next_spawn: 0.0,
                left_side: true,
            }],
        }
    }

    pub fn default_desert() -> Self {
        Self {
            channels: vec![
                ScatterChannel {
                    material: "cactus",
                    interval: 2.0,
                    lateral_range: 4.0,
                    color: [255, 255, 255, 255],
                    scalar: 1.0,
                    placement: Placement::Random,
                    flip_mode: FlipMode::Random,
                    motion: ScatterMotion::default(),
                    next_spawn: 0.0,
                    left_side: true,
                },
                ScatterChannel {
                    material: "tumbleweed",
                    interval: 5.0,
                    lateral_range: 5.0,
                    color: [255, 255, 255, 255],
                    scalar: 1.0,
                    placement: Placement::Fixed(true),
                    flip_mode: FlipMode::BySide,
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
