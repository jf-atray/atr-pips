use glam::{Quat, Vec2, Vec3, Vec4};

use crate::assets::AssetRegistry;
use crate::brushes::Brush;
use crate::gamescope::game::PipMaker;
use crate::spacial::transform::Transform;
use crate::tables::domain::Domain;

pub struct SpriteSpawn {
    pub name: String,
    pub xform: Transform,
    pub scale: Vec2,
    pub color: Vec4,
}

pub struct Scene {
    pub spawns: Vec<SpriteSpawn>,
}

pub struct SceneAccess {
    pub current: Scene,
}

impl Scene {
    pub fn demo(registry: &AssetRegistry, _pixels_per_unit: f32) -> Self {
        let mut spawns = Vec::with_capacity(64);
        let mut rng = rand::rng();

        for i in 0..64u32 {
            let x = (i % 8) as f32 * 0.8 - 2.8;
            let y = (i / 8) as f32 * 0.8 - 2.8;
            let name = registry.pick_random(&mut rng).to_string();

            spawns.push(SpriteSpawn {
                name,
                xform: Transform {
                    xyz: Vec3::new(x, y, 0.0),
                    rot: Quat::IDENTITY,
                },
                scale: Vec2::ONE,
                color: Vec4::ONE,
            });
        }

        Self { spawns }
    }

    pub fn load(&self, registry: &AssetRegistry, domain: &mut Domain) {
        for spawn in &self.spawns {
            let entry = registry.get(&spawn.name);
            let pip = PipMaker {
                xform: spawn.xform.clone(),
                brush: Brush {
                    canvas: entry.canvas,
                    material: entry.material,
                    scale: spawn.scale,
                    color: spawn.color,
                },
            };
            domain.make(pip);
        }
    }
}
