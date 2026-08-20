use glam::{Quat, Vec3, Vec4};

use crate::brushes::Brush;
use crate::gather::impls::gather_mut;
use crate::scripting::context::DomainView;
use crate::scripting::script::Script;
use crate::spacial::motion::Motion;
use crate::spacial::transform::Transform;
use crate::ecs::PipId;
use crate::ecs::scope::Scope;

const CLOUD_SPRITES: &[(&str, f32)] = &[("cloud_1", 4.0), ("cloud_2", 1.0)];

const CLOUD_DRIFT_SPEED: f32 = 0.075;
const CLOUD_WRAP_X: f32 = 6.0;
const CLOUD_COUNT: usize = 4;
const CLOUD_Z: f32 = 0.9;

pub struct CloudDriftSystem {
    clouds: Vec<PipId>,
    initialized: bool,
}

impl CloudDriftSystem {
    pub fn new() -> Self {
        Self {
            clouds: Vec::new(),
            initialized: false,
        }
    }

    fn spawn_clouds(&mut self, ctx: &mut DomainView) {
        for _ in 0..CLOUD_COUNT {
            let x = (rand::random::<f32>() * 2.0 - 1.0) * CLOUD_WRAP_X;
            let y = 0.5 + rand::random::<f32>() * 1.0;
            let w = 1.5 + rand::random::<f32>() * 1.5;
            let (name, aspect) =
                CLOUD_SPRITES[rand::random::<u32>() as usize % CLOUD_SPRITES.len()];
            let h = w / aspect;

            let name = name.to_string();
            let sprite = ctx
                .asset_registry
                .get(&name)
                .unwrap_or(ctx.asset_registry.get("__white__").unwrap());
            let pip = ctx.domain.make(|scope: &mut Scope| {
                let mut brush = Brush::from_sprite(sprite);
                brush.scale =
                    Vec3::new(w / sprite.natural_scale.x, h / sprite.natural_scale.y, 1.0);
                brush.color = Vec4::ONE;
                scope.core.with(
                    Transform {
                        xyz: Vec3::new(x, y, CLOUD_Z),
                        rot: Quat::IDENTITY,
                    },
                    brush,
                    name,
                    Motion::default(),
                );
            });
            self.clouds.push(pip);
        }
    }
}

impl Script for CloudDriftSystem {
    fn update(&mut self, ctx: &mut DomainView) {
        if !self.initialized {
            self.spawn_clouds(ctx);
            self.initialized = true;
        }

        for &pip in &self.clouds {
            if let Some(xform) =
                gather_mut(&ctx.domain.ids, &mut ctx.domain.tables.core.xforms, pip)
            {
                xform.xyz.x += CLOUD_DRIFT_SPEED * ctx.dt;
                if xform.xyz.x > CLOUD_WRAP_X {
                    xform.xyz.x = -CLOUD_WRAP_X;
                }
            }
        }
    }
}
