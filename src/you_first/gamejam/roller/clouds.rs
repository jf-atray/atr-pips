use glam::{Quat, Vec3, Vec4};
use std::collections::HashMap;

use crate::addition::Pips;
use crate::assets::SpriteEntry;
use crate::brushes::Brush;
use crate::ecs::core::CoreWorld;
use crate::ecs::PipId;
use crate::ecs::scope::Scope;
use crate::gather::impls::gather_mut;
use crate::input::Input;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;

const CLOUD_SPRITES: &[(&str, f32)] = &[("cloud_1", 4.0), ("cloud_2", 1.0)];

const CLOUD_DRIFT_SPEED: f32 = 0.075;
const CLOUD_WRAP_X: f32 = 12.0;
const CLOUD_COUNT: usize = 4;
const CLOUD_Z: f32 = 0.9;

#[derive(Debug)]
pub struct CloudDriftSystem {
    clouds: Vec<PipId>,
}

impl CloudDriftSystem {
    pub fn new() -> Self {
        Self { clouds: Vec::new() }
    }

    fn spawn_clouds(
        &mut self,
        pips: &mut Pips,
        asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        self.clouds.reserve(CLOUD_COUNT);
        for _ in 0..CLOUD_COUNT {
            let x = (rand::random::<f32>() * 2.0 - 1.0) * CLOUD_WRAP_X;
            let y = 0.5 + rand::random::<f32>() * 1.0;
            let w = 1.5 + rand::random::<f32>() * 1.5;
            let (name, aspect) =
                CLOUD_SPRITES[rand::random::<u32>() as usize % CLOUD_SPRITES.len()];
            let h = w / aspect;

            let name = name.to_string();
            let sprite = asset_registry
                .get(&name)
                .unwrap_or(asset_registry.get("__white__").unwrap());
            let pip = pips.make(|scope: &mut Scope| {
                let mut brush = Brush::from_sprite(sprite);
                brush.scale =
                    Vec3::new(w / sprite.natural_scale.x, h / sprite.natural_scale.y, 1.0);
                brush.color = Vec4::ONE;
                scope
                    .view::<CoreWorld>()
                    .unwrap()
                    .with(
                        Transform {
                            xyz: Vec3::new(x, y, CLOUD_Z),
                            rot: Quat::IDENTITY,
                        },
                        brush,
                        name,
                        Motion {
                            vel: Vec3::new(CLOUD_DRIFT_SPEED, 0.0, 0.0),
                        },
                    );
            });
            self.clouds.push(pip);
        }
    }
}

impl Solver for CloudDriftSystem {}

impl CloudDriftSystem {
    pub fn update(
        &mut self,
        _dt: f32,
        pips: &mut Pips,
        _scripts: &mut crate::addition::ScriptsMap,
        _signals: &mut crate::addition::SignalsMap,
        _input: &Input,
        asset_registry: &HashMap<String, SpriteEntry>,
    ) {
        if self.clouds.is_empty() {
            self.spawn_clouds(pips, asset_registry);
        }

        for &pip in &self.clouds {
            if let Some(xform) = gather_mut(&pips.ids, &mut pips.tables.core.xforms, pip)
                && xform.xyz.x > CLOUD_WRAP_X
            {
                xform.xyz.x = -CLOUD_WRAP_X;
            }
        }
    }
}
