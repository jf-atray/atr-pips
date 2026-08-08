use glam::{Quat, Vec3};
use rand::Rng;

use crate::brushes::Brush;
use crate::gamescope::game::PipMaker;
use crate::spacial::transform::Transform;
use crate::tables::domain::Domain;
use crate::tables::{CanvasId, MaterialId};

pub struct Scene {
    pub pips: Vec<PipMaker>,
}

pub struct SceneAccess {
    pub current: Scene,
}

impl Scene {
    pub fn demo(canvas: CanvasId, materials: &[MaterialId]) -> Self {
        let mut pips = Vec::with_capacity(64);
        let mut rng = rand::rng();

        for i in 0..64u32 {
            let x = (i % 8) as f32 * 0.2 - 0.7;
            let y = (i / 8) as f32 * 0.2 - 0.7;
            let material = materials[rng.random_range(0..materials.len())];

            pips.push(PipMaker {
                xform: Transform {
                    xyz: Vec3::new(x, y, 0.0),
                    rot: Quat::IDENTITY,
                },
                brush: Brush { canvas, material },
            });
        }

        Self { pips }
    }

    pub fn load(&self, domain: &mut Domain) {
        for pip in &self.pips {
            domain.make(pip.clone());
        }
    }
}
